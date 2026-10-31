// The light/dark switch of sites with dark mode (#146). Optional: without
// JavaScript the page follows the system's setting (prefers-color-scheme,
// in style.css) and the switch stays hidden.
//
// Loaded in <head>, not deferred: it sets the visitor's choice
// (data-mode="light" or "dark" on <html>) before the page shows, so it
// doesn't flash up in the other colors first. The choice is kept in the
// browser (localStorage) for every page of the site; "system" keeps none.
(function () {
  "use strict";
  var KEY = "qed-mode";
  var root = document.documentElement;

  function saved() {
    try {
      return window.localStorage.getItem(KEY);
    } catch (e) {
      return null;
    }
  }

  function keep(choice) {
    try {
      if (choice === "system") window.localStorage.removeItem(KEY);
      else window.localStorage.setItem(KEY, choice);
    } catch (e) {
      // Storage switched off: the choice lasts for this page.
    }
  }

  function apply(choice) {
    if (choice === "light" || choice === "dark") root.setAttribute("data-mode", choice);
    else root.removeAttribute("data-mode");
  }

  function systemIsDark() {
    return !!window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  // "light" or "dark": what the page shows now. The player asks, to start
  // in the same colors.
  window.qedMode = function () {
    return root.getAttribute("data-mode") || (systemIsDark() ? "dark" : "light");
  };

  apply(saved());

  // The switch: three buttons, light, the system's setting and dark; the
  // current choice is pressed.
  function setUp() {
    var group = document.getElementById("mode-switch");
    if (!group) return;
    var buttons = group.querySelectorAll("button[data-choice]");
    function show(choice) {
      buttons.forEach(function (button) {
        button.setAttribute("aria-pressed", String(button.getAttribute("data-choice") === choice));
      });
    }
    show(root.getAttribute("data-mode") || "system");
    buttons.forEach(function (button) {
      button.addEventListener("click", function () {
        var choice = button.getAttribute("data-choice");
        apply(choice);
        show(choice);
        keep(choice);
      });
    });
    group.hidden = false;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", setUp);
  } else {
    setUp();
  }
})();
