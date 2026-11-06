+++
title = "Pictures and files"
description = "Add photos and logos to Media, files to download to Downloads, and files the website needs as they are to Static; use them on pages."
menu = "main"
weight = 30
summary = "Add photos and logos, files to download, and files the website needs as they are."
+++

A site keeps its files in three places, each with a tab of its own:

:::: columns
::: column
title = "Media"
boxed = true
:::

:icon-photo: Pictures: photos, logos, illustrations. JPEG, PNG, GIF,
WebP and SVG, and ICO for the [site icon](site-settings.md).

::: column
title = "Downloads"
boxed = true
:::

:icon-download: Files for visitors to download: PDFs, documents, video
files. Any type.

::: column
title = "Static"
boxed = true
:::

:icon-folder: Files the website needs exactly as they are, at their own
address: a presentation, a verification file.
::::

## Adding files

**Import images…** (or **Import files…**) copies files from your
computer into the site. q.e.d. gives them clean names: lower case, no
spaces, so they work as web addresses. The originals stay where they
were. **Show in file manager** opens the folder itself.

Each file shows its size and which pages use it; files nobody uses are
marked **Not used**. Only those can be deleted (:icon-trash:), so no page
ends up with a missing picture.

## Pictures on pages

The **Image**, **Text with image**, **Gallery** and **Hero** blocks pick
a picture from Media with a click. In text, a picture is written as
`![what it shows](/media/photo.jpg)`.

Always fill in the **alternative text**: what the picture shows, in a
sentence. People who can't see it hear it read aloud, and search engines
read it too.

::: text-image
text = """
**Large photos are no problem.** q.e.d. makes smaller copies of every
photo when it builds the site, and each visitor's browser loads the one
that fits their screen. A phone doesn't download the 4000 pixels of a
camera photo to show 400.

Large pictures open big when clicked, over the page, without leaving it."""
image = "manual/media-tab.png"
alt = "The Media tab of q.e.d.: a search field, the Import images button, and the site's pictures as tiles, each with its name, size, pixel size and the pages that use it."
image_side = "right"
:::

## Where pictures are cut

Galleries, tiles in post and page lists, and the Hero block show
pictures cut to fit, a Gallery as squares. q.e.d. cuts around the
middle, which suits most pictures. When it doesn't, say for a portrait
that loses the head, choose the spot yourself: click the picture in the
Media tab to show it big, click :icon-crop: **Crop**, then click the
spot that must stay visible. A square beside it shows the result, and
the choice counts everywhere the picture is used. :icon-focus-centered:
**Middle** goes back to cutting around the middle.

## Files to download

The **Download** block offers one file from Downloads, with its type and
size. In text, link it as `[the menu](/assets/menu.pdf)`. A **Video**
block can play a video file from Downloads, too.

## The whole site as a PDF

:icon-file-type-pdf: **Make PDF of the site** in the Downloads tab puts
the whole site into one PDF, to print, to archive or to offer for
download. It starts with a title page and a table of contents of the
top two levels of pages, with page numbers; PDF readers show them as
bookmarks, too. Then come the pages in the order of the site, each
top-level page on a new page, and after them the posts and episodes,
oldest first, in a chapter of their own after the page that lists them
(such as the Blog), so they read forward like a diary.

The PDF is plain black text on white, without the site's colors, fonts
or logo. Pictures, tables, formulas, charts and columns come along;
links to other pages jump inside the PDF, and links to other websites
show their address in a footnote, so they can be read on paper. The
player and videos become a link to their page on the website.

It's made only when you click, and lands in Downloads like any other
file: `my-site.pdf`, named after the site. Offer it with a **Download**
block. A site in several languages gets one PDF per language
(`my-site-en.pdf`). Click again after changing pages to bring it up to
date; drafts and posts dated in the future stay out, and so do pages
whose **Include in the PDF of the site** setting is off.

::: faq
[[items]]
question = "My PDFs are in Media. Is that a problem?"
answer = "No, they keep working. Media is meant for pictures, though; **Move to Downloads** in the Media tab moves them and updates the pages that link to them."

[[items]]
question = "Can I use a picture from another website?"
answer = "Only if you may, and then better as a copy in Media than as a link: q.e.d. sites load nothing from other websites, so they stay fast and don't hand your visitors' data to anyone."
:::
