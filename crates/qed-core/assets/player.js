// Starts the Podlove Web Player on episode pages (#049). Optional: without
// JavaScript the page keeps its plain <audio> player and chapter list.
//
// The page has <div data-podlove-player data-episode="player.json">; the
// player's settings for the whole site are in podlove-player/config.json,
// next to the player's files (see assets/podlove-player/README.md). This
// file sits at the site root, so it finds both from its own address.
(function () {
  "use strict";
  var script = document.currentScript;
  if (!script) return;
  var root = new URL("./", script.src);

  function start() {
    var targets = document.querySelectorAll("[data-podlove-player]");
    if (!targets.length || typeof window.podlovePlayer !== "function") return;
    fetch(new URL("podlove-player/config.json", root))
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
          });
        });
      })
      .catch(function () {
        // Without the player, the plain one stays.
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
