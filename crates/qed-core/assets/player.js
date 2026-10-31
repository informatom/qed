// Starts the Podlove Web Player on episode pages (#049). Optional: without
// JavaScript the page keeps its plain <audio> player and chapter list.
//
// Loaded in <head>, not deferred: while the player loads, everything
// marked data-player-fallback is hidden already, so the plain player
// doesn't flash up first. If the player fails, or takes too long, it
// comes back.
//
// The page has <div data-podlove-player data-episode="player.json">; the
// player's settings for the whole site are in podlove-player/config.json,
// next to the player's files (see assets/podlove-player/README.md). This
// file sits at the site root, so it finds both from its own address.
(function () {
  "use strict";
  var script = document.currentScript;
  // One start finds every player on the page (the Latest episode block
  // on an episode page makes two); should this load twice, once is enough.
  if (!script || window.qedPlayerStarting) return;
  window.qedPlayerStarting = true;
  var root = new URL("./", script.src);

  var page = document.documentElement;
  var style = document.createElement("style");
  // While it starts: no plain player, and the room the player will take
  // (heights from the generator, narrow below 600 px), so the page
  // doesn't jump when it's there.
  style.textContent =
    ".player-starting [data-player-fallback] { display: none !important; }" +
    ".player-starting [data-podlove-player] { min-height: var(--player-narrow); }" +
    "@container (min-width: 600px) {" +
    "  .player-starting [data-podlove-player] { min-height: var(--player-wide); }" +
    "}";
  document.head.appendChild(style);
  page.classList.add("player-starting");
  // The plain player again, if the real one isn't there by then.
  function giveUp() {
    page.classList.remove("player-starting");
  }
  var waiting = window.setTimeout(giveUp, 10000);

  function start() {
    var targets = document.querySelectorAll("[data-podlove-player]");
    if (!targets.length || typeof window.podlovePlayer !== "function") {
      giveUp();
      return;
    }
    // A site with dark mode has settings for it too (mode.js says which).
    var dark = typeof window.qedMode === "function" && window.qedMode() === "dark";
    fetch(new URL(dark ? "podlove-player/config-dark.json" : "podlove-player/config.json", root))
      .then(function (response) {
        return response.json();
      })
      .then(function (config) {
        // The player loads the rest of itself from here.
        config.base = new URL("podlove-player/", root).href;
        targets.forEach(function (target) {
          var episode = new URL(target.getAttribute("data-episode"), document.baseURI).href;
          Promise.resolve(window.podlovePlayer(target, episode, config)).then(function () {
            // The player is there: the plain player and chapter list go.
            document.querySelectorAll("[data-player-fallback]").forEach(function (element) {
              element.hidden = true;
            });
            window.clearTimeout(waiting);
            giveUp();
          });
        });
      })
      .catch(function () {
        // Without the player, the plain one stays.
        window.clearTimeout(waiting);
        giveUp();
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
