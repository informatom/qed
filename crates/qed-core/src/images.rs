//! Responsive images: smaller copies of the site's photos, so a phone
//! doesn't download a 4000 pixel original to show it 400 pixels wide.
//!
//! The generator asks [`Images::info`] about every image a page shows. That
//! reads only the file header and decides which widths the image gets
//! ([`WIDTHS`], only those clearly smaller than the original). The page then
//! lists them in `srcset`, and the browser picks one. After all pages are
//! rendered, [`Images::write`] makes the copies.
//!
//! Resizing is slow, so every copy is kept in `.qed-cache/images/` in the
//! site folder and reused while the original stays the same. The cache
//! folder ignores itself in git and can be deleted at any time.
//!
//! The copies keep the original's format: JPEG stays JPEG, PNG stays PNG.
//! GIF (often animated) and WebP (the image crate can only write lossless
//! WebP, which is larger) are left as they are.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::UNIX_EPOCH;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use sha2::{Digest, Sha256};

/// The widths of the smaller copies, in pixels: phones, the content column,
/// and the content column on sharp screens (and hero backgrounds).
pub const WIDTHS: [u32; 3] = [480, 960, 1600];

/// The cache folder, inside the site folder.
pub const CACHE_DIR: &str = ".qed-cache";

/// Good enough to look the same as the original at a third of the size.
const JPEG_QUALITY: u8 = 82;

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("can't write {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("can't resize {}: {source}", path.display())]
    Image {
        path: PathBuf,
        source: image::ImageError,
    },
}

/// What a page needs to know about an image in `media/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInfo {
    /// The size as shown, after applying the camera's rotation.
    pub width: u32,
    pub height: u32,
    /// The widths of the smaller copies, smallest first; empty if there are
    /// none.
    pub widths: Vec<u32>,
}

/// The images used by the pages of one build.
///
/// Rendering only has `&Images` (shared, read-only access), but each call to
/// `info` remembers the image. `RefCell` allows that: it moves Rust's "one
/// writer or many readers" check from compile time to run time.
pub struct Images {
    media_dir: PathBuf,
    used: RefCell<BTreeMap<String, Option<ImageInfo>>>,
}

impl Images {
    pub fn new(media_dir: &Path) -> Self {
        Images {
            media_dir: media_dir.to_path_buf(),
            used: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn media_dir(&self) -> &Path {
        &self.media_dir
    }

    /// The size and the copy widths of `name` (a path inside `media/`), or
    /// `None` if it isn't a readable image. Reads the file once per build.
    pub fn info(&self, name: &str) -> Option<ImageInfo> {
        if let Some(known) = self.used.borrow().get(name) {
            return known.clone();
        }
        let info = read_info(&self.media_dir.join(name));
        self.used
            .borrow_mut()
            .insert(name.to_string(), info.clone());
        info
    }

    /// Writes the smaller copies of every image asked about into
    /// `out_media` (the built site's `media/`), from the cache where
    /// possible. Cached copies no image used anymore are deleted. Returns the
    /// number of copies written.
    pub fn write(&self, out_media: &Path, cache_dir: &Path) -> Result<usize, ImageError> {
        let cache = cache_dir.join("images");
        let mut jobs = Vec::new();
        for (name, info) in self.used.borrow().iter() {
            let Some(info) = info.as_ref().filter(|info| !info.widths.is_empty()) else {
                continue;
            };
            let source = self.media_dir.join(name);
            let key = cache_key(name, &source)?;
            let extension = extension_of(name);
            let copies = info
                .widths
                .iter()
                .map(|&width| Resized {
                    width,
                    cached: cache.join(format!("{key}-{width}.{extension}")),
                    target: out_media.join(copy_name(name, width)),
                })
                .collect();
            jobs.push(Job { source, copies });
        }

        let missing: Vec<&Job> = jobs
            .iter()
            .filter(|job| job.copies.iter().any(|resized| !resized.cached.is_file()))
            .collect();
        if !missing.is_empty() {
            create_cache(cache_dir, &cache)?;
            in_parallel(&missing, |job| resize(&job.source, &job.copies))?;
        }

        let mut kept = BTreeSet::new();
        for resized in jobs.iter().flat_map(|job| &job.copies) {
            if let Some(parent) = resized.target.parent() {
                fs::create_dir_all(parent).map_err(io(parent))?;
            }
            fs::copy(&resized.cached, &resized.target).map_err(io(&resized.target))?;
            kept.insert(resized.cached.clone());
        }

        prune(&cache, &kept)?;
        Ok(kept.len())
    }
}

/// One original and the copies it gets.
struct Job {
    source: PathBuf,
    copies: Vec<Resized>,
}

struct Resized {
    width: u32,
    /// Where the copy is kept between builds.
    cached: PathBuf,
    /// Where it goes in the built site.
    target: PathBuf,
}

/// Runs `work` on every item, on as many threads as the computer has cores.
/// Each thread takes the next item until none are left. Returns the first
/// error, after all threads have finished.
///
/// `std::thread::scope` guarantees the threads end before it returns, so
/// they may borrow `items` and `work` from this function.
fn in_parallel<T: Sync>(
    items: &[T],
    work: impl Fn(&T) -> Result<(), ImageError> + Sync,
) -> Result<(), ImageError> {
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism()
        .map_or(1, |cores| cores.get())
        .min(items.len());
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    while let Some(item) = items.get(next.fetch_add(1, Ordering::Relaxed)) {
                        work(item)?;
                    }
                    Ok(())
                })
            })
            .collect();
        // A thread that panicked passes the panic on, as if it had happened
        // here.
        handles.into_iter().try_for_each(|handle| {
            handle
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
        })
    })
}

/// `logos/a.png` at 480 pixels is `logos/a.480w.png`. Imported media files
/// never have a dot in their name before the extension, so this can't clash
/// with one of them.
pub fn copy_name(name: &str, width: u32) -> String {
    match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.ends_with('/') => format!("{stem}.{width}w.{extension}"),
        _ => format!("{name}.{width}w"),
    }
}

fn read_info(path: &Path) -> Option<ImageInfo> {
    let reader = ImageReader::open(path).ok()?.with_guessed_format().ok()?;
    let format = reader.format()?;
    let mut decoder = reader.into_decoder().ok()?;
    let (width, height) = decoder.dimensions();
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let (width, height) = if turns_sideways(orientation) {
        (height, width)
    } else {
        (width, height)
    };

    let resizable = matches!(format, ImageFormat::Jpeg | ImageFormat::Png);
    // A copy at 95% of the original's width would save almost nothing.
    let widths = WIDTHS
        .into_iter()
        .filter(|&copy| resizable && u64::from(copy) * 10 < u64::from(width) * 9)
        .collect();
    Some(ImageInfo {
        width,
        height,
        widths,
    })
}

/// Phone cameras store photos sideways and note the rotation in the file
/// (EXIF). Browsers apply it, so the size and the copies must too.
fn turns_sideways(orientation: Orientation) -> bool {
    matches!(
        orientation,
        Orientation::Rotate90
            | Orientation::Rotate270
            | Orientation::Rotate90FlipH
            | Orientation::Rotate270FlipH
    )
}

/// Identifies one version of an original: a new file, or a changed one,
/// gets a new key. Name, size and modification time are enough and much
/// faster than reading the whole file.
fn cache_key(name: &str, source: &Path) -> Result<String, ImageError> {
    let metadata = fs::metadata(source).map_err(io(source))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |time| time.as_nanos());
    let hash = Sha256::digest(format!("{name}\n{}\n{modified}", metadata.len()));
    Ok(hash[..8].iter().map(|byte| format!("{byte:02x}")).collect())
}

fn extension_of(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, extension)| extension.to_lowercase())
        .unwrap_or_default()
}

/// Creates the cache folder with a `.gitignore` that ignores everything in
/// it, itself included, so the cache never ends up in the site's history.
fn create_cache(cache_dir: &Path, cache: &Path) -> Result<(), ImageError> {
    fs::create_dir_all(cache).map_err(io(cache))?;
    let ignore = cache_dir.join(".gitignore");
    if !ignore.is_file() {
        fs::write(&ignore, "# q.e.d.'s cache; safe to delete.\n*\n").map_err(io(&ignore))?;
    }
    Ok(())
}

/// Decodes the original once and writes a copy for each width, largest
/// first; each smaller copy is made from the previous one, which is faster
/// than starting from the full-size original every time.
fn resize(source: &Path, copies: &[Resized]) -> Result<(), ImageError> {
    let failed = |source_error| ImageError::Image {
        path: source.to_path_buf(),
        source: source_error,
    };
    let reader = ImageReader::open(source)
        .map_err(io(source))?
        .with_guessed_format()
        .map_err(io(source))?;
    let format = reader.format();
    let mut decoder = reader.into_decoder().map_err(failed)?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder).map_err(failed)?;
    image.apply_orientation(orientation);

    let mut copies: Vec<&Resized> = copies.iter().collect();
    copies.sort_by_key(|resized| std::cmp::Reverse(resized.width));
    for resized in copies {
        let height = scaled_height(image.width(), image.height(), resized.width);
        image = image.resize_exact(resized.width, height, FilterType::CatmullRom);
        let target = &resized.cached;
        // Write under a temporary name and rename, so an interrupted build
        // never leaves half a file in the cache.
        let partial = target.with_extension("partial");
        let file = fs::File::create(&partial).map_err(io(&partial))?;
        let mut writer = BufWriter::new(file);
        match format {
            Some(ImageFormat::Jpeg) => JpegEncoder::new_with_quality(&mut writer, JPEG_QUALITY)
                .encode_image(&DynamicImage::ImageRgb8(image.to_rgb8())),
            _ => image.write_to(&mut writer, ImageFormat::Png),
        }
        .map_err(failed)?;
        // Dropping a BufWriter would write its last bytes but ignore an
        // error doing so (a full disk, say); flushing reports it.
        writer.flush().map_err(io(&partial))?;
        fs::rename(&partial, target).map_err(io(target))?;
    }
    Ok(())
}

/// The height that keeps the aspect ratio at `new_width`, at least 1 pixel.
pub(crate) fn scaled_height(width: u32, height: u32, new_width: u32) -> u32 {
    let scaled =
        (u64::from(height) * u64::from(new_width) + u64::from(width) / 2) / u64::from(width);
    u32::try_from(scaled).unwrap_or(u32::MAX).max(1)
}

/// Deletes cached copies this build didn't use.
fn prune(cache: &Path, kept: &BTreeSet<PathBuf>) -> Result<(), ImageError> {
    let Ok(entries) = fs::read_dir(cache) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && !kept.contains(&path) {
            fs::remove_file(&path).map_err(io(&path))?;
        }
    }
    Ok(())
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> ImageError {
    let path = path.to_path_buf();
    move |source| ImageError::Io { path, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    /// Writes a `width` × `height` test image (format from the extension).
    fn photo(dir: &Path, name: &str, width: u32, height: u32) {
        let image = RgbImage::from_fn(width, height, |x, y| {
            Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        image.save(dir.join(name)).unwrap();
    }

    #[test]
    fn names_copies_next_to_the_original() {
        assert_eq!(copy_name("team.jpg", 480), "team.480w.jpg");
        assert_eq!(copy_name("logos/a.png", 960), "logos/a.960w.png");
        assert_eq!(copy_name("README", 480), "README.480w");
    }

    #[test]
    fn makes_only_widths_clearly_smaller_than_the_original() {
        let dir = tempfile::tempdir().unwrap();
        photo(dir.path(), "big.jpg", 2000, 1000);
        photo(dir.path(), "medium.png", 1000, 500);
        photo(dir.path(), "small.jpg", 400, 300);
        let images = Images::new(dir.path());

        assert_eq!(images.info("big.jpg").unwrap().widths, vec![480, 960, 1600]);
        // 960 would be 96% of 1000.
        assert_eq!(images.info("medium.png").unwrap().widths, vec![480]);
        assert_eq!(images.info("small.jpg").unwrap().widths, Vec::<u32>::new());
        assert_eq!(images.info("missing.jpg"), None);
    }

    #[test]
    fn writes_copies_and_reuses_the_cache() {
        let site = tempfile::tempdir().unwrap();
        let media = site.path().join("media");
        fs::create_dir(&media).unwrap();
        photo(&media, "photo.jpg", 2000, 1000);
        let out = site.path().join("_site/media");
        let cache = site.path().join(CACHE_DIR);

        let images = Images::new(&media);
        images.info("photo.jpg");
        assert_eq!(images.write(&out, &cache).unwrap(), 3);
        assert_eq!(
            image::image_dimensions(out.join("photo.960w.jpg")).unwrap(),
            (960, 480)
        );
        assert!(cache.join(".gitignore").is_file());

        // A second build copies from the cache instead of resizing again.
        let cached: Vec<PathBuf> = fs::read_dir(cache.join("images"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(cached.len(), 3);
        let before = fs::metadata(&cached[0]).unwrap().modified().unwrap();
        let images = Images::new(&media);
        images.info("photo.jpg");
        images.write(&out, &cache).unwrap();
        assert_eq!(
            fs::metadata(&cached[0]).unwrap().modified().unwrap(),
            before
        );

        // A build that no longer uses the photo clears its copies out.
        Images::new(&media).write(&out, &cache).unwrap();
        assert_eq!(fs::read_dir(cache.join("images")).unwrap().count(), 0);
    }

    #[test]
    fn turns_sideways_phone_photos_upright() {
        use image::ImageEncoder;

        let site = tempfile::tempdir().unwrap();
        let media = site.path().join("media");
        fs::create_dir(&media).unwrap();
        // Stored 2000 wide and 1000 high, with EXIF saying "rotate by 90°",
        // as phone cameras do: a minimal little-endian TIFF block with one
        // entry, Orientation (0x0112) = 6.
        let exif = vec![
            0x49, 0x49, 42, 0, 8, 0, 0, 0, // header, first entry list at 8
            1, 0, // one entry
            0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, // Orientation = 6
            0, 0, 0, 0, // no further entry lists
        ];
        let pixels = RgbImage::new(2000, 1000);
        let mut encoder = JpegEncoder::new(fs::File::create(media.join("phone.jpg")).unwrap());
        encoder.set_exif_metadata(exif).unwrap();
        encoder
            .write_image(pixels.as_raw(), 2000, 1000, image::ExtendedColorType::Rgb8)
            .unwrap();

        let images = Images::new(&media);
        let info = images.info("phone.jpg").unwrap();
        assert_eq!((info.width, info.height), (1000, 2000));
        assert_eq!(info.widths, vec![480]);

        let out = site.path().join("_site/media");
        images.write(&out, &site.path().join(CACHE_DIR)).unwrap();
        assert_eq!(
            image::image_dimensions(out.join("phone.480w.jpg")).unwrap(),
            (480, 960)
        );
    }
}
