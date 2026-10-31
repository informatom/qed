+++
title = "Menus"
description = "The main menu and the footer menu: made from the pages by themselves, or arranged by hand with groups and links; in the top bar or a sidebar."
menu = "main"
weight = 34
summary = "The main and footer menus, by themselves or by hand."
+++

A site has two menus: the **main menu** at the top of every page, and
the **footer menu** at the bottom.

## Made from the pages

Usually there's nothing to do. Each page's **Settings** tab says where
it goes:

- **Menu**: **Main menu**, **Footer menu**, or **Not in a menu**.
- **Order**: a number; pages with smaller numbers come first.
- **Icon**: shown next to the page in the menu (see [Icons](icons.md)).

Pages below another page in the menu (a blog's posts, the pages below
*About*) go with it: in the top bar it becomes a dropdown with the page
itself first and the others below it; a long one scrolls. In the
sidebar they fold (see below).

New pages are in no menu until you choose one. The footer menu is
always made this way; the imprint and privacy pages belong there.

## In a sidebar

When there are more pages than fit into the top bar, switch on **Show
the main menu in a sidebar** in the **Menu** tab. The menu then runs
down the left side of every page; on phones it becomes a *Menu* box
above the content. It's saved at once.

A page with pages below it folds them away in the sidebar: a click on
its name opens the page, a click on the arrow beside it shows or hides
the pages below. Only the part of the menu with the page a visitor is
on starts open.

## Arranged by hand

To group pages, give them other titles in the menu, or add links to
other websites, press **Arrange the menu by hand** in the **Menu** tab.
q.e.d. starts with the menu as it is, and you change it:

:::: columns
::: column
title = "Page"
boxed = true
:::

:icon-file-text: A page of the site, with its title or one of your own
for the menu.

::: column
title = "Link"
boxed = true
:::

:icon-link: A title and an address: another website
(`https://…`) or an address on your site (`/blog`).

::: column
title = "Group"
boxed = true
:::

:icon-list: A title with pages and links below it: a dropdown in the
top bar. In the sidebar it folds open and closed with a click; only the
group of the page a visitor is on starts open. A **Line** inside a group
divides it.
::::

The arrows move an entry up and down, :icon-trash: removes it. **Save
menu** saves it; **Back to automatic** returns to the menu made from the
pages' settings.

A page that is a draft, or was deleted, isn't shown in the menu. Its
entry stays in the Menu tab with a warning, and building the website
says so, until you pick another page or remove the entry.

::: faq
[[items]]
question = "I made a new page, but it's not in the menu."
answer = "With the automatic menu, set its **Menu** to **Main menu** in the page's Settings tab. With a menu arranged by hand, add it in the Menu tab: new pages don't go there by themselves."

[[items]]
question = "Can a group have groups inside?"
answer = "No, one level only. That keeps the menu easy to use on phones."
:::
