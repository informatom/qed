+++
title = "Backups"
description = "How to back up a q.e.d. site and edit it on two computers: push and pull its git repository, and what that leaves out."
menu = "main"
weight = 50
redirect_from = ["/backups"]
+++

A q.e.d. site is a folder, and every site folder is a
[git](https://git-scm.com) repository: q.e.d. records each version there
(see the *Versions* tab). So a backup is a copy of that repository, and
the best copy is one on another computer, made with `git push`. It holds
not just today's state but every version, and you can bring any of them
back.

## Setting up a backup

Once per site, create an empty repository somewhere else: on
[Codeberg](https://codeberg.org) or [GitHub](https://github.com) (a
private repository is fine), or on your own server
(`git init --bare /var/git/my-site.git`). Then, in the *Versions* tab
under *Push and pull*:

1. Add the repository's SSH address (on Codeberg or GitHub: the “SSH”
   address under *Clone*), for example
   `git@codeberg.org:me/my-site.git`.
2. *Make a key*, copy it and add it on the server: in the repository's
   settings as a *deploy key* with write access, or on your own server
   in `~/.ssh/authorized_keys`. The private key stays in your
   computer's keychain.
3. *Push*. The first time, q.e.d. shows the server's key fingerprint;
   trust it if it matches what the server's owner publishes.

## Making a backup

*Push* in the Versions tab. It closes the open version first (with the
note in the *Save a version* field, if you typed one), so the latest
changes are part of the backup. Publishing closes the open version too,
so pushing after each publish is a good habit.

## Two computers

On the second computer, choose *Get a site from git…* in the site menu
(or on the start screen), enter the same address and make a key there
too (one key per computer). q.e.d. copies the site with all its
versions. From then on: push when you're done on one computer, pull
before you start on the other. When a site opens, q.e.d. asks the
remote whether it has new versions and offers to pull them.

If you edited on both computers without pulling in between, *Pull*
merges the versions as long as different files changed. If the same
page changed on both, nothing changes and q.e.d. names the files: keep
one side, or fix it with git.

## Restoring

On any computer, *Get a site from git…*, or on the command line:

```sh
git clone git@codeberg.org:me/my-site.git my-site
```

Open the folder in q.e.d. and it's all there: pages, media, files to
download, static files, fonts, settings and the full version history.

## With git on the command line

The remotes are git's own, so `git remote add backup …` and
`git push backup main` work as well and show up in the app. q.e.d.
pushes and pulls only sites that are a git repository of their own; for
a site folder inside a larger repository, use git there.

## What isn't in the backup

- **Publish targets** (`targets.toml`) are deliberately not part of the
  history. After restoring, set up the targets again under *Publishing*.
  They only hold server names and folders, nothing secret.
- **Passwords and SSH keys** live in your computer's keychain, never in
  the site folder. Back them up the way you back up your other
  passwords, or make a new key on a new computer and add it to the
  server.
- **The built site** (`_site/`) and q.e.d.'s image cache (`.qed-cache/`)
  are left out on purpose: the next build makes them again.

## Without git

Copying the whole site folder (including the hidden `.git` folder) to a
backup drive works too, and keeps the history as well. It's just easier
to forget than a push after publishing.
