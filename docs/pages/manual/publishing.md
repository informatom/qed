+++
title = "Publishing"
description = "Put the website on your own webspace over SFTP: set up a target once, test the connection, then publish with a note on what's new."
menu = "main"
weight = 60
summary = "Put the website on your webspace: set up a target once, then publish with a click."
+++

q.e.d. builds the website on your computer and copies it to your
webspace, the space for a website your hosting company gives you. It
uses SFTP, a secure way of copying files that almost every hosting
company offers. Nothing runs on the server: the website is plain files,
fast and cheap to host.

## Before the first time

Your hosting company gives you four things, usually in its customer
area or in the e-mail from when you signed up:

- the **server**, such as `ssh.example.com`
- your **user** name
- a **password**, or the option to log in with a key
- the **folder on the server** the website goes in, such as `/www` or
  `/home/you/public_html`

Also enter the site's **web address** in the **Site** tab: feeds, link
previews and the sitemap need it.

## Set up a target

A *target* is a place the site is published to. In the **Publish** tab,
:icon-plus: **Add target** and fill in the four things, with a **Name**
for it, such as *Website*. With **Log in with: Key**, q.e.d. makes a key
for this server; copy it into your hosting company's settings for SSH
keys (or the server's `~/.ssh/authorized_keys`). Then **Save target**.

Passwords and keys are kept in your computer's keychain, never in the
site folder, so they don't end up in backups or on other computers.

## Test, then publish

::: image
image = "manual/publish-tab.png"
alt = "The Publish tab: a Build button, and under Publishing one target named Alpha with the badges key and server trusted, and the buttons Edit, Remove, Test connection and Publish. The target's address is blurred."
caption = "A target that's been tested: **server trusted**, ready to **Publish**."
:::

1. **Test connection.** The first time, q.e.d. shows the server's key
   fingerprint, its proof of identity. If your hosting company publishes
   it, compare; then **Trust**. From now on q.e.d. notices if a
   different server answers at that address.
2. :icon-upload: **Publish.** q.e.d. asks *What's new on the site?*:
   a few words for the history, such as *Summer opening hours*. Then it
   builds the site and copies only what changed since last time.
3. **Open the website** to look at it online.

Files q.e.d. didn't upload itself are never touched on the server, so
other things in the same folder stay as they are.

::: faq
[[items]]
question = "Do I have to build first?"
answer = "No. Publish builds the site by itself. :icon-hammer: **Build** in the Publish tab is for looking at the files, or uploading them yourself."

[[items]]
question = "Can I see the website before it goes online?"
answer = "Yes: the :icon-eye: button in the top right corner shows it in your browser, from your computer only."

[[items]]
question = "Publish waits for an episode. Why?"
answer = "On a podcast site, every published episode needs its audio file, in every format the podcast offers, or podcast apps would get a broken episode. Add the audio, or mark the episode as a draft until it's ready."

[[items]]
question = "Is the old website gone when I publish?"
answer = "q.e.d. replaces what it put there before and adds what's new. Files it didn't put there stay. Every publish is also marked in the site's history, so you can see what was online when."
:::
