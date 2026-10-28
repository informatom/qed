// The embedded player (#128): podlove-player/share.html, which the
// player's embed code puts into other sites as an iframe, loads this
// before Podlove's share.js. It does what the original share.html did
// in an inline script, and two things more:
//
// - Only this site's episodes and settings: share.html?episode=… with an
//   address elsewhere would make this site show someone else's player.
// - The site's player settings (podlove-player/config.json, the theme's
//   colors) when the embed code doesn't name any.
//
// Podlove's share.js reads `episode`, `config` and `t` from the address,
// so this fixes the address before it runs.
(function () {
  "use strict";
  // Where share.js finds the rest of the player, next to share.html.
  window.resourceBaseUrl = "5.13.0/player/";
  var script = document.currentScript;
  if (!script) return;
  var params = new URLSearchParams(window.location.search);
  var ours = function (name) {
    var value = params.get(name);
    if (!value) return false;
    try {
      return new URL(value, window.location.href).origin === window.location.origin;
    } catch (error) {
      return false;
    }
  };
  if (!ours("episode")) params.delete("episode");
  if (!ours("config")) params.set("config", new URL("config.json", script.src).href);
  window.history.replaceState(null, "", "?" + params.toString());
})();
