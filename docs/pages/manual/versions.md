+++
title = "Versions"
description = "Every save is kept: the open version, versions with a note, going back to an earlier state of a page or of the whole site, and bringing back deleted pages."
menu = "main"
weight = 40
summary = "Every save is kept; go back to an earlier state of a page or of the whole site."
+++

q.e.d. keeps the history of your site, every save of it, in the site
folder. Nothing you saved is lost, and anything can be undone.

## The open version

:icon-history: Your saves go into the **open version**: one version that
grows with each save while you work. The **Versions** tab shows it at the
top, marked *open*, with when it started and when you last saved.

It's closed, and becomes a version of its own, when:

- you give it a note and click **Close version**: say in a few words
  what you did, such as *New opening hours*;
- you publish the site (Publish asks what's new, and that becomes the
  note);
- you back it up or bring in changes from another computer (push or
  pull, see [Backups](backups.md));
- you come back after a break: after an hour without saves, or on the
  next day, q.e.d. asks for a note for what you did before. **Use
  suggested note** names it after its times, such as *Saves on 27 Oct,
  13:50–21:12*.

So each working session becomes one version you can go back to.

::: image
image = "manual/versions-tab.png"
alt = "The Versions tab of a site: on the left a field for a note and the Save version button, and Push and pull; on the right the list of versions with their notes, such as Saves on 27 Oct, 21:12–21:25 and Before moving /datenschutz, each with a Go back button, and below it Deleted pages with a Restore button."
caption = "The **Versions** tab: versions with their notes, **Go back** for the whole site, and **Deleted pages**."
:::

## Going back

:::: columns
::: column
title = "One page"
boxed = true
:::

Open the page and its **Versions** tab. Pick a version to see what it
changed, and :icon-history: **Restore this version** to put it into the editor. Nothing is saved
until you click **Save**, so you can look first.

::: column
title = "The whole site"
boxed = true
:::

:icon-arrow-back-up: **Go back** next to a version in the **Versions** tab puts
every page, picture and file back the way it was then. The present is
saved as a version first, so you can come back to it the same way.
::::

## Deleted pages

Under **Deleted pages**, the **Versions** tab lists pages that were
deleted, with the last version that still had them. :icon-arrow-back-up:
**Restore** brings one back as it was before it was deleted.

::: call-to-action
headline = "A copy on another computer"
text = "The history is in the site folder. If the computer breaks, it's gone with it. A backup on another computer keeps every version safe."
button_label = "Backups"
button_link = "backups.md"
:::
