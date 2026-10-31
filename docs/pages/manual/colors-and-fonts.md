+++
title = "Colors and fonts"
description = "The Theme tab: pick a theme, decide which of its colors goes where, choose fonts, or make colors of your own and keep them under a name."
menu = "main"
weight = 36
summary = "Pick a theme, place its colors, choose fonts, make your own colors."
+++

How the website looks is set in the **Theme** tab, for a light and,
if you like, a dark side. Everything there is
saved as you pick it; preview the site again (:icon-eye:) to see it. On
the right, a small sketch of a page shows the result while you choose.

## A theme

At the bottom of the tab are the themes, under **Light** and **Dark**,
each with its colors in a row. Click one, and the whole site takes its
colors. Every theme has four colors (**primary**, **secondary**,
**accent** and **neutral**) on its background.

## Which color goes where

Under **Colors**, decide for each part of the page which of the theme's
colors it gets:

- **Header**: **Plain** (the page background) or a colored bar.
- **Links** and **Buttons**: one of the four colors.
- **Hero**, **Call to action**, **Boxed columns**, **Content
  background** and **Footer**: **plain**, **shaded** (a shade darker
  than the page) or one of the four colors. The content background
  takes only a light tint, so text stays readable.

On a dark theme the footer is **shaded** unless you pick another color:
the theme's accent is often a pale color that would make a bright band
below a dark page. On a light theme it's **accent**.

**Reset** puts the header and every color back to the defaults.

## Light and dark mode

Many people set their computer or phone to dark colors, at least in the
evening. Switch on **Light and dark mode** at the top of the tab, and
the website gets a second look for them: a dark theme with colors of
its own. Visitors then see the side that matches their system. Three
small buttons next to the search in the header (a sun, a computer and a
moon) let them pick light, their system's setting, or dark; the browser
remembers the choice for the whole site.

When you switch it on, the dark side starts with a dark theme that suits
the light one (*catppuccin-mocha* for *catppuccin-latte*, *dark* for
*light*) and with the same color choices. **Light** and **Dark** next to
the switch decide which side the rest of the tab changes: the theme,
the header, the colors and the custom colors each have their own on
both sides. Saved colors can be used on either side. The fonts are the
same for both.

Switching it off again removes the dark side's settings; the light side
stays as it was.

## Fonts

**Text** and **Headings** each have **Change…**, which searches some
2,000 open fonts (the Google Fonts among them). Pick one, and q.e.d.
downloads it once into the site; the website then serves it itself, so
no visitor's browser asks Google for anything. Headings use the text
font unless you pick another; **Use system font** goes back to the font
of the visitor's computer.

## Your own colors

For colors of your own, such as those of a logo, press **Customize**
next to the theme's name under **Custom colors**:

- **Start from**: the theme whose colors you change.
- **Background**, **Primary**, **Secondary**, **Accent**, **Neutral**: a
  color picker each. Text on each color is made readable by itself.
- **Corners**: **Sharp**, **Soft**, **Round**, or as the theme has them.
- **Reset to …** takes all colors from the starting theme again.

Under **Saved colors**, give them a name and press **Save these
colors**. Then you can try other themes and come back with one click.

**An example.** The running podcast [3 Schweinehunde](https://3-schweinehun.de/)
starts from *light* and changes it: a mint green (`#48cfad`) as primary
and accent, a lighter one as secondary, dark grey as neutral, and soft
corners, saved as "3 Schweinehunde". The header bar is primary, the
footer shaded, and text and headings use the font *Ubuntu*.

::: image
image = "manual/theme-tab.png"
alt = "The Theme tab of q.e.d. for the site 3 Schweinehunde: the color choices for header, links, buttons and the other parts of the page, the font Ubuntu, the custom mint colors saved as 3 Schweinehunde, the grid of themes below, and on the right a sketch of a page showing where each color goes."
caption = "The **Theme** tab of 3 Schweinehunde: on the right, where each color goes."
:::

::: faq
[[items]]
question = "I picked a theme, but the website looks the same."
answer = "The preview shows the site as it was built. Press :icon-eye: again to build it with the new colors."

[[items]]
question = "Can visitors switch between light and dark?"
answer = "Yes, with **Light and dark mode** at the top of the Theme tab: a second, dark theme for visitors whose system is set to dark, and a switch in the header. Without it the site has the one theme you choose."

[[items]]
question = "What do visitors without JavaScript see?"
answer = "The side that matches their system's setting. Only the switch needs JavaScript; without it, it isn't shown."
:::
