# Podlove Web Player

The [Podlove Web Player](https://github.com/podlove/podlove-ui/tree/development/apps/player)
(`@podlove/web-player`, MIT license), version **5.13.0** (build of
2024-12-07, revision dd0459fe638811edc210f44e36882724d36055f8, as the
`*.LICENSE.txt` files say). q.e.d. copies this folder into a built site
that has episodes with audio, at `podlove-player/`, so nothing loads
from podlove.org. The player itself loads nothing from other servers;
the only outside addresses in it are its share tab's links, and q.e.d.
leaves that tab off.

## Where this copy comes from

Taken on 2026-10-28 from jekyll-octopod-bulma
(<https://github.com/jekyll-octopod/jekyll-octopod-bulma>, folder
`assets/podlove-player/`, commit 78d958f of 2026-08-12), which had
vendored the player's built files. Changed on the way in: the `.gz` and
`.map` files are left out. The `*.LICENSE.txt` files stay: they hold the
licenses of the libraries bundled into the scripts.

## How q.e.d. uses it

- `crates/qed-core/src/generator.rs` builds this folder into the program
  (`PODLOVE_PLAYER`, with `include_dir`) and writes it into the built
  site (`write_player`), with `podlove-player/config.json`: the player
  settings for the whole site (`version: 5`, the theme's colors as
  `theme.tokens`, the chapters tab open).
- Each episode with audio gets `player.json` next to its page: the
  episode in the player's format (title, show, poster, audio and
  downloads per format, chapters, contributors, and the transcript's
  cues, which q.e.d. reads from the episode's WebVTT file; see
  `podcast::transcript_cues`).
- `assets/player.js` (q.e.d.'s own, in the folder above) finds
  `[data-podlove-player]` on the page, reads `config.json`, sets `base`
  to this folder's full address, and calls
  `window.podlovePlayer(element, episodeUrl, config)`. Once the player is
  there, it hides everything marked `data-player-fallback`: the plain
  `<audio>` player and the chapter list, which are what readers without
  JavaScript get.
- `embed.js` loads the rest (`5.13.0/player/…`, `variant-*.js`) itself,
  relative to `base`, so the folder must stay together.

The data formats are documented at
<https://docs.podlove.org/podlove-web-player/v5/configuration>.

## Updating it

1. Get the built files of a newer 5.x release: `npm pack @podlove/web-player`
   (or the release on GitHub) and take its `dist/` folder: `embed.js`,
   `variant-*.js` and the versioned `x.y.z/player/` folder.
2. Replace this folder's contents with them; leave out `.gz` and `.map`
   files, keep the `*.LICENSE.txt` files and this README, and update the
   version and "Where this copy comes from" above.
3. Check the episode and config formats in `generator.rs` against the
   new version's documentation.
4. `cargo test -p qed-core`, then build a podcast site and play an
   episode in a browser: audio, chapters, transcript, the theme's colors.
