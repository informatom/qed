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

## Files to download

The **Download** block offers one file from Downloads, with its type and
size. In text, link it as `[the menu](/assets/menu.pdf)`. A **Video**
block can play a video file from Downloads, too.

::: faq
[[items]]
question = "My PDFs are in Media. Is that a problem?"
answer = "No, they keep working. Media is meant for pictures, though; **Move to Downloads** in the Media tab moves them and updates the pages that link to them."

[[items]]
question = "Can I use a picture from another website?"
answer = "Only if you may, and then better as a copy in Media than as a link: q.e.d. sites load nothing from other websites, so they stay fast and don't hand your visitors' data to anyone."
:::
