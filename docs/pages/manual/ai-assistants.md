+++
title = "AI assistants"
description = "Let an AI assistant such as Claude work on your site: what it can and can't do, setting it up in Claude Desktop or Claude Code (terminal or VS Code), and how to check and undo its changes."
menu = "main"
weight = 75
summary = "Let an AI assistant such as Claude write and change pages for you, with q.e.d.'s rules."
+++

An AI assistant such as Claude can work on your site for you. You
describe what you want in your own words, and it does it in q.e.d.:

- "Write a page about our team, with a photo of each of us."
- "Move all posts from 2023 below the archive page."
- "Check every page for typos and fix them."
- "Which pictures aren't used anywhere?"

It works on the same site you have open in q.e.d., following the same
rules as the app. You see every change in :icon-history: **Versions**
and can undo it.

## What it can do, and what it can't

:::: columns
::: column
title = "It can"
boxed = true
:::

- read, write and make pages and posts, with all blocks
- give a page a new address (links and redirects are taken care of, as
  when you move a page in q.e.d.)
- delete pages that nothing links to, and pictures and files that
  nothing uses any more (to replace screenshots, for example)
- add pictures and files from your computer, and files the website
  needs exactly as they are (such as old slide shows) at their old
  addresses
- change the footer, shared blocks and the site settings (menu, logo,
  theme, web address, your profiles elsewhere); the site has to open
  with them, or q.e.d. refuses
- build the site and report broken links
- save a version with a note saying what it changed

::: column
title = "It can't"
boxed = true
:::

- publish: putting the site online is always your click, in
  :icon-upload: **Publish**
- save a broken page: q.e.d. checks every page first and tells the
  assistant what's wrong
- overwrite a change you made meanwhile
- touch anything outside the site's folder
::::

## Setting it up

1. Open the site in q.e.d. and go to the :icon-settings: **App** tab.
   Under **AI assistants** you find the settings for this site, ready
   to copy.
2. For **Claude Desktop**: in Claude Desktop, open Settings →
   Developer → Edit Config. This opens the file
   `claude_desktop_config.json`. Paste the part q.e.d. shows (**Copy**),
   save, and restart Claude Desktop. If the file already has servers
   in it, add q.e.d.'s entry to the ones there.
3. For **Claude Code in a terminal**: copy the command q.e.d. shows and
   run it once in a terminal.
4. For **Claude Code in VS Code**: the command from step 3 often doesn't
   work there, because the VS Code extension brings its own Claude Code
   without the `claude` command. Instead, save the part q.e.d. shows
   under *Claude Code in VS Code* as a file named `.mcp.json` in the
   folder you open in VS Code, for example the site's folder. When
   Claude Code starts in that folder, it asks once whether to use the
   server in it; say yes.

Do this once for each site you want the assistant to work on. The
settings name q.e.d. itself: the assistant starts it in the
background, without a window, and talks to it. Other assistants that
speak the [Model Context Protocol](https://modelcontextprotocol.io)
work the same way.

If you move q.e.d. or the site's folder, set it up again: the settings
contain both places.

## Checking and undoing its work

The assistant's changes are saved like yours. Ask it to "save a version
when you're done": its note then shows in :icon-history: **Versions**,
for example *"Team page with photos"*. There you see what changed on
each page and can go back, as described in [Versions](versions.md).

If you edit a page in q.e.d. while the assistant changes the same page,
nothing is lost. The assistant is told that the page changed and reads
it again; q.e.d. asks you which version to keep when you save.

To see the result before publishing, use the :icon-eye: preview button
in q.e.d. as always.

::: faq
heading = "Questions"

[[items]]
question = "Does the assistant send my site to the internet?"
answer = "The assistant itself runs at its maker, for Claude at Anthropic. The pages it reads and writes go there as part of the conversation, like anything you type into it. q.e.d. doesn't send anything anywhere."

[[items]]
question = "Does q.e.d. have to be open?"
answer = "No. The assistant starts q.e.d. in the background by itself. If q.e.d. is open too, you see its changes when you look at the page again."

[[items]]
question = "Can it publish by mistake?"
answer = "No. There is no way for it to publish. You decide when the site goes online."
:::
