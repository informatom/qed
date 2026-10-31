+++
title = "q.e.d."
description = "Eine Desktop-App für Ihre eigene Website oder Podcast-Seite: ganz in der App gemacht, ohne Programmieren, und auf Ihrem eigenen Webspace veröffentlicht."
menu = "main"
+++

::: image
image = "mascot-banner.jpeg"
alt = "Der q.e.d.-Roboter an einem Schreibtisch, mit einem Mikrofon und einem Tablet, auf dem eine Website zu sehen ist. Darüber: q.e.d., Desktop-App, Static Site Generator, Podcast Publishing."
:::

::: hero
headline = "Ihre Website, ohne Programmieren"
text = "Schreiben Sie Ihre Seiten, fügen Sie Bilder und Episoden hinzu, wählen Sie Farben und Schriften: alles in der App. q.e.d. macht daraus eine schnelle Website und veröffentlicht sie auf Ihrem eigenen Webspace. Kein Code, kein Server, kein Abo."
button_label = "Zum Handbuch"
button_link = "/de/manual"
:::

q.e.d. ist Open Source und noch in früher Entwicklung. Es entsteht
öffentlich mit [Claude Code](https://claude.com/claude-code), in Rust,
Tauri, Svelte und TypeScript.

Auf Deutsch gibt es hier diese Startseite und das
[Handbuch](manual/index.md). **Die [englische Seite](/) hat mehr**: den
[Blog](/blog) über die Entwicklung, das [Changelog](/changelog), den
[Backlog](/backlog) und die Hinweise zur [Entwicklung](/development).

## Für Podcaster

q.e.d. macht auch Podcast-Websites und behält alles auf Ihrem eigenen
Webspace: kein Hosting-Dienst, kein Tracking, nichts wird von anderswo
geladen.

- **Episoden** mit Staffel, Nummer, Kapiteln, Mitwirkenden und
  Transkripten; die Shownotes schreiben Sie wie jede andere Seite.
- **Feeds** für jedes Audioformat, das Sie anbieten (MP3, M4A, Ogg,
  Opus, FLAC), mit den Tags von Apple und Podcasting 2.0. War Ihre alte
  Seite mit jekyll-octopod gemacht, funktionieren die Apps Ihrer
  Hörerinnen und Hörer weiter.
- **Der Podlove Web Player** mit Kapiteln, Transkript, Playlist und
  Teilen, und der **Podlove Subscribe Button**, beide auf Ihrem eigenen
  Webspace.
- **Mitwirkende** tragen Sie einmal ein; jede und jeder bekommt eine
  Seite mit den eigenen Episoden.

Diese Seite ist selbst eine q.e.d.-Seite: eine `site.toml` und ein
Ordner `pages/`, im Ordner `docs/` des
[q.e.d.-Repositorys](https://github.com/informatom/qed).

## Download

Die Installer der neuesten Version sind auf der
[Releases-Seite](https://github.com/informatom/qed/releases/latest). Sie
sind noch nicht signiert, daher fragt Ihr System vor dem ersten Start
einmal nach.

:::: columns
::: column
title = "macOS"
:::

Laden Sie die `.dmg` herunter, öffnen Sie sie und ziehen Sie qed in den
Ordner Programme. Es läuft auf Macs mit Apple Silicon und Intel.

Öffnen Sie qed beim ersten Mal. macOS meldet, dass es die App nicht
prüfen kann: wählen Sie **Fertig**. Öffnen Sie dann
Systemeinstellungen → **Datenschutz & Sicherheit**, scrollen Sie nach
unten zum Hinweis zu qed, wählen Sie **Dennoch öffnen** und bestätigen
Sie mit Ihrem Passwort. Danach öffnet sich qed wie gewohnt.

Unter macOS 14 und älter klicken Sie stattdessen mit der rechten
Maustaste auf qed in Programme und wählen **Öffnen**.

::: column
title = "Windows"
:::

Laden Sie die `.exe` herunter und starten Sie sie.

Zeigt Windows „Der Computer wurde durch Windows geschützt“, wählen Sie
**Weitere Informationen** und dann **Trotzdem ausführen**.

::: column
title = "Linux"
:::

Unter Debian und Ubuntu laden Sie die `.deb` herunter und installieren
sie mit

```sh
sudo apt install ./qed_*.deb
```

Auf jeder anderen Distribution laden Sie das `.AppImage` herunter,
machen es ausführbar (`chmod +x qed_*.AppImage`) und starten es.
::::
