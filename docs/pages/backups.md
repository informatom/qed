+++
title = "Backups"
description = "How to back up a q.e.d. site: push its git repository to a remote, and what that leaves out."
menu = "footer"
weight = 5
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
(`git init --bare /var/git/my-site.git`). Then, in the site folder:

```sh
git remote add backup git@codeberg.org:me/my-site.git
git push -u backup main
```

## Making a backup

Close the open version in the *Versions* tab first (give it a note), so
the latest changes are part of the history. Then:

```sh
git push backup main
```

Publishing closes the open version too, so pushing after each publish is
a good habit. A *Back up* button in the app is planned (#109).

## Restoring

On any computer:

```sh
git clone git@codeberg.org:me/my-site.git my-site
```

Open the folder in q.e.d. and it's all there: pages, media, files to
download, static files, fonts, settings and the full version history.

## What isn't in the backup

- **Publish targets** (`targets.toml`) are deliberately not part of the
  history. After restoring, set up the targets again under *Publishing*.
  They only hold server names and folders, nothing secret.
- **Passwords and SSH keys** live in your computer's keychain, never in
  the site folder. Back them up the way you back up your other
  passwords, or make a new key on a new computer and add it to the
  server.
- **Changes since the last version.** Until a version is closed, the
  newest changes are only in the folder. Close the version before
  pushing.
- **The built site** (`_site/`) and q.e.d.'s image cache (`.qed-cache/`)
  are left out on purpose: the next build makes them again.

## Without git

Copying the whole site folder (including the hidden `.git` folder) to a
backup drive works too, and keeps the history as well. It's just easier
to forget than a push after publishing.
