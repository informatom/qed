# qed backlog

Open work and open decisions. Items keep their number for good; when one is
done or decided, move it to **Done / decided** with a short note.

## To decide

- **#001 Upload storage.** Where uploaded audio and images live: local disk
  (Docker volume) or S3-compatible storage. What upload size limit to use.
- **#002 Registration.** Open sign-up, invite-only, or accounts created by an
  admin? Should the first user become the instance admin? Does the demo
  instance need different rules?
- **#003 Email.** Pick an email service (SMTP or an API) for account confirmation
  and password reset. Should confirmation be required before signing in? It
  currently is: `confirm_on_create`.
- **#004 Site templates.** What the generated sites look like and how they're
  themed. Which CSS: Bulma like jekyll-octopod, Tailwind or daisyUI, or plain?
  Can users edit templates, and in which language (HEEx/EEx, or Liquid via
  `solid`)?
- **#005 Web player.** Use the self-hosted Podlove Web Player, as in
  jekyll-octopod?
- **#006 Podcast-level Apple/Podcasting 2.0 fields.** Podcast type (episodic or
  serial), owner name and email (`itunes:owner`), cover art, funding, guid.
- **#007 Episode-level fields.** Season, episode type (full, trailer or bonus),
  episode image, guid stability when the slug changes.
- **#008 Transcripts.** Model WebVTT transcripts per episode, as jekyll-octopod
  does.
- **#009 Audio formats.** Should users upload every format themselves (mp3,
  m4a, ogg, opus), or should qed transcode from one master file with ffmpeg?
- **#010 Duration and size detection.** Read duration and byte size from
  uploads automatically (ffprobe) or enter them by hand?
- **#011 GitHub Pages authentication.** Personal access token (as modelled now),
  a deploy key (reusing the generated SSH key), or a GitHub App?
- **#012 SSH transfer method.** Run `rsync` (adds rsync and openssh-client to the
  Docker image), or SFTP with Erlang's built-in `:ssh` (no extra binaries, but
  no delta transfer)?
- **#013 Background jobs.** Run publishing in the background, e.g. with
  Oban/AshOban (SQLite engine), with progress shown live?
- **#014 Roles.** Is owner/editor enough, or is a read-only viewer role needed?
- **#015 Global podcast slugs.** Podcast slugs are currently unique across the
  whole instance. Keep that, or make them unique per user?
- **#016 qed's license.** For example AGPL-3.0 (protects the hosted-service
  case) or MIT.
- **#017 Demo instance.** Where it's hosted, whether and when data is reset,
  and abuse limits.
- **#018 Offline drafting.** Should shownotes be draftable offline in the PWA
  and synced later? Server-side generation stays either way.
- **#019 Comments and sharing.** Carry over jekyll-octopod's share options?
  Comments were dropped there; is that final?

## To build

- **#020 Home page.** Replace the Phoenix landing page with the app's home: the
  user's podcasts and a sign-in call to action.
- **#021 Podcast UI.** List, create and edit podcasts (LiveView with
  `AshPhoenix.Form`).
- **#022 Episode editor.** Use the Markdown editor from
  `/daten/DATA/phoenix/mercator` for shownotes. Include a chapter editor.
- **#023 Audio files UI.** Upload with LiveView uploads, or enter an external
  URL. One entry per format.
- **#024 Members UI.** Invite or add members, change roles, remove members.
- **#025 Publish targets UI.** Create and edit targets. For SSH, show the public
  key ready to copy, plus a "regenerate key" button.
- **#026 Site generator.** Render HTML pages (index, episode pages, imprint).
  Generate one RSS feed per audio format with iTunes, Podcasting 2.0 and
  Podlove Simple Chapters tags, plus paged feeds. Copy assets and audio.
- **#027 Publisher: ZIP.** Build the site and offer it as a download.
- **#028 Publisher: SSH.** Deploy to the document root, honouring
  `rsync_delete`.
- **#029 Publisher: GitHub Pages.** Push the built site to the configured
  branch.
- **#030 SSH host key check.** Pin the server's host key on first connect and
  warn when it changes.
- **#031 Connection test.** A "test connection" button for SSH and GitHub
  targets.
- **#032 Publish history.** Record each publish run per target: time, user,
  result and log.
- **#033 Cover art and images.** Upload, resize and validate images (Apple
  requires 1400–3000 px square).
- **#034 Feed import.** Import an existing podcast from its RSS feed or from a
  jekyll-octopod site (compare `octopod import`).
- **#035 Last-owner protection.** Prevent removing or demoting a podcast's last
  owner.
- **#036 Docker image tools.** Add whatever the publishers need (rsync,
  openssh-client, git, ffmpeg), depending on #009, #012 and #029.
- **#037 SQLite backups.** Document or automate backups of the `qed_data`
  volume.
- **#038 CLOAK_KEY rotation.** Document how to rotate the encryption key, and
  that losing it makes stored secrets unreadable.
- **#039 AGENTS.md.** The generated AGENTS.md says to avoid daisyUI, which
  contradicts the chosen stack. Update it.

## Done / decided

- Stack: Elixir, Phoenix 1.8, Ash 3, SQLite, Tailwind and daisyUI. Ship as
  source, Docker Compose, and a hosted demo usable as a PWA.
- The server does the work. The PWA is an installable front end for a running
  instance.
- Multi-user: podcasts have members with roles (owner, editor).
- Audio can be uploaded to qed or point to an external URL.
- SSH targets use a key pair generated by qed. The private key is stored
  encrypted (AshCloak).
- Data model: podcast, membership, episode (Markdown shownotes, chapters),
  audio file (per format), publish target (SSH, ZIP, GitHub Pages).
