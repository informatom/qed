+++
title = "Lizenz und Dank"
description = "q.e.d. ist freie Software unter der GPL; Ihre Website und Ihre Inhalte bleiben Ihre. Was eine gebaute Website von anderen Projekten enthält, unter welchen Lizenzen, und wem q.e.d. dankt."
menu = "main"
weight = 90
summary = "Freie Software; Ihre Inhalte bleiben Ihre; die Teile von anderen."
+++

## q.e.d. ist freie Software

q.e.d. steht unter der
[GNU General Public License, Version 3 oder später](https://www.gnu.org/licenses/gpl-3.0.html).
Sie dürfen es für alles verwenden, kostenlos, auch für geschäftliche
Websites; weitergeben; und ändern, solange das, was Sie weitergeben,
unter derselben Lizenz bleibt. Der Quellcode liegt auf
[GitHub](https://github.com/informatom/qed).

## Ihre Website gehört Ihnen

Die GPL gilt für das Programm, nicht für das, was Sie damit machen. Ihre
Seiten, Bilder, Episoden und die Website, die q.e.d. daraus baut,
gehören Ihnen, zu Bedingungen, die Sie wählen.

Sagen Sie, welche: für einen Podcast im Feld **Copyright or license**
des Tabs Podcast (etwa `CC BY 4.0`), und auf der Website in der
Fußzeile, zum Beispiel als Zeile mit einem Link auf die Lizenz. 3
Schweinehunde verwendet, wie viele Podcasts,
[Creative Commons BY 4.0](https://creativecommons.org/licenses/by/4.0/deed.de):
Alle dürfen die Episoden teilen und weiterverwenden, wenn sie den
Podcast nennen.

## Was Ihre Website von anderen enthält

Eine von q.e.d. gebaute Website enthält ein paar Dateien aus anderen
Open-Source-Projekten. Alle dürfen auf jeder Website verwendet werden,
auch auf kommerziellen; Lizenzhinweise, die ihre Autoren in die Dateien
geschrieben haben, bleiben dort.

| Teil | Wo er verwendet wird | Lizenz |
| --- | --- | --- |
| [Podlove Web Player](https://podlove.org/podlove-web-player/) 5.13.0 | Episoden eines Podcasts | MIT |
| [Podlove Subscribe Button](https://podlove.org/podlove-subscribe-button/) | der Block Subscribe | MIT |
| [Pagefind](https://pagefind.app/) | die Suche | MIT |
| Icons von [Lucide](https://lucide.dev) | Icons in Text und Menüs | ISC |
| [Tailwind CSS](https://tailwindcss.com/) und [daisyUI](https://daisyui.com/) | das Stylesheet und die Themes | MIT |
| [Catppuccin](https://catppuccin.com/) | die vier Catppuccin-Themes | MIT |
| [Latin Modern Math](https://www.gust.org.pl/projects/e-foundry/lm-math) | mathematische Formeln | GUST Font License |
| Schriften von [Fontsource](https://fontsource.org/) | die Schriften, die Sie wählen | die der jeweiligen Schrift, meist die SIL Open Font License |

Die Lizenz einer Schrift steht neben ihr im Stylesheet der Website.

## Dank

q.e.d. steht auf den Schultern vieler Projekte. Besonders:

- **[Podlove](https://podlove.org/)**: der Web Player und der Subscribe
  Button, und jahrelange Arbeit am offenen Podcasting.
- **[jekyll-octopod](https://jekyll-octopod.github.io/)**, der Vorgänger
  von q.e.d. für Podcasts, der auf [Octopod](https://github.com/pattex/octopod)
  von Arne Eilermann aufbaut. Seine Feed- und Audioadressen leben in
  q.e.d. weiter, daher ziehen Podcasts um, ohne einen Abonnenten zu
  verlieren.
- **[Tauri](https://tauri.app/)** und **[Rust](https://www.rust-lang.org/)**,
  aus denen die App gemacht ist, und **[Svelte](https://svelte.dev/)**
  für ihre Oberfläche.
- **[libgit2](https://libgit2.org/)**, das die Versionen bewahrt,
  **[Lucide](https://lucide.dev)**, **[daisyUI](https://daisyui.com/)**,
  **[Pagefind](https://pagefind.app/)**, **[Fontsource](https://fontsource.org/)**
  und **[Catppuccin](https://catppuccin.com/)**.

Der q.e.d.-Roboter und das Logo wurden von Googles Bildmodell Nano
Banana 2 nach der Beschreibung von Stefan Haslinger gezeichnet; das Logo
wurde dann von Hand als SVG nachgezeichnet, damit es in jeder Größe
scharf bleibt
([A logo in SVG](https://qed.informatom.com/blog/2026-10-22-a-logo-in-svg/),
auf Englisch).

Das Programm selbst ist aus mehr als 750 Rust-Bibliotheken und ein paar
JavaScript-Paketen gebaut, fast alle unter der MIT- oder
Apache-Lizenz. [Bibliotheken](libraries.md) listet jede, mit Version und
Lizenz (auf Englisch). q.e.d. wird mit
[Claude Code](https://claude.com/claude-code) geschrieben; der
[Blog](https://qed.informatom.com/blog/) erzählt, wie (auf Englisch).

## Fragen, Fehler und Ideen

Etwas funktioniert nicht oder fehlt? Sagen Sie es auf
[GitHub](https://github.com/informatom/qed/issues): eine kurze
Beschreibung, was Sie erwartet haben, und die Version aus dem Tab
**App**. Fehler und Lücken in diesem Handbuch sind dort ebenfalls
willkommen.
