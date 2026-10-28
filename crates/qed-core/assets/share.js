// q.e.d.'s share buttons on episode pages (#053). Optional: without
// JavaScript the buttons stay hidden and the plain links (Bluesky, email)
// still work. Nothing is loaded from anywhere else.
//
// Each button carries what it needs in data- attributes:
//   data-share="copy"      copies data-url
//   data-share="native"    opens the device's share menu (if it has one)
//   data-share="mastodon"  asks once for the reader's Mastodon server,
//                          remembers it, and opens that server's share page
// data-text is the text to share, data-ask the question for the server,
// data-done what the copy button says after copying.
(function () {
  "use strict";
  var KEY = "qed-mastodon-server";

  function remembered() {
    try {
      return localStorage.getItem(KEY) || "";
    } catch (e) {
      return "";
    }
  }

  function remember(server) {
    try {
      localStorage.setItem(KEY, server);
    } catch (e) {
      // Private windows may refuse; then it asks again next time.
    }
  }

  document.querySelectorAll("[data-share]").forEach(function (button) {
    var kind = button.getAttribute("data-share");
    var url = button.getAttribute("data-url");
    var text = button.getAttribute("data-text") || "";

    if (kind === "native" && !navigator.share) return;
    if (kind === "copy" && !(navigator.clipboard && navigator.clipboard.writeText)) return;
    button.hidden = false;

    button.addEventListener("click", function () {
      if (kind === "native") {
        navigator.share({ title: text, url: url }).catch(function () {});
      } else if (kind === "copy") {
        navigator.clipboard.writeText(url).then(function () {
          var label = button.textContent;
          button.textContent = button.getAttribute("data-done") || label;
          setTimeout(function () {
            button.textContent = label;
          }, 2000);
        });
      } else if (kind === "mastodon") {
        var answer = window.prompt(button.getAttribute("data-ask") || "Mastodon server", remembered());
        if (!answer) return;
        // "@me@fosstodon.org", "https://fosstodon.org/" or "fosstodon.org".
        var server = answer.trim().replace(/^https?:\/\//, "").replace(/^.*@/, "").replace(/\/.*$/, "");
        if (!server) return;
        remember(server);
        window.open("https://" + server + "/share?text=" + encodeURIComponent(text + " " + url), "_blank", "noopener");
      }
    });
  });
})();
