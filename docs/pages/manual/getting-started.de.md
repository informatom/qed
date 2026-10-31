+++
title = "Erste Schritte"
description = "q.e.d. installieren, eine neue Website anlegen oder eine öffnen, sich in der App zurechtfinden und das Ergebnis im Browser ansehen."
menu = "main"
weight = 10
summary = "q.e.d. installieren, eine Website anlegen oder öffnen, sich zurechtfinden, das Ergebnis ansehen."
+++

## Installieren

Die Installer sind auf der
[Releases-Seite](https://github.com/informatom/qed/releases/latest):
eine `.dmg` für macOS, eine `.exe` für Windows, eine `.deb` und ein
`.AppImage` für Linux. Die [q.e.d.-Website](https://qed.informatom.com/de/)
erklärt, wie Sie an der ersten Warnung Ihres Systems vorbeikommen.
q.e.d. sucht bei jedem Start nach einer neueren Version und fragt, bevor
es eine installiert.

::: call-to-action
headline = "Ihre Website ist ein Ordner"
text = "Alles, woraus eine Website besteht, liegt in einem Ordner auf Ihrem Computer: ihre Seiten, Bilder, Einstellungen und ihre Geschichte. Hinter q.e.d. stehen kein Konto und kein Server; der Ordner ist die Website. Legen Sie ihn zu Ihren anderen Dokumenten und sichern Sie ihn wie diese."
button_label = "Sicherungen"
button_link = "backups.md"
:::

## Eine Website anlegen oder öffnen

Ist keine Website geöffnet, bietet q.e.d. drei Wege:

:::: columns
::: column
title = "New site…"
boxed = true
:::

:icon-file-plus: Fragt nach dem Titel der Website, ihrer Sprache und wo
ihr Ordner liegen soll. q.e.d. legt den Ordner mit einer Startseite an
und beginnt ihre Geschichte.

::: column
title = "Open site folder…"
boxed = true
:::

:icon-folder: Öffnet eine Website, die schon auf Ihrem Computer liegt,
hier oder anderswo mit q.e.d. gemacht.

::: column
title = "Get a site from git…"
boxed = true
:::

:icon-git-branch: Holt eine Website von einem Server, auf dem sie
gesichert wurde; siehe [Sicherungen](backups.md).
::::

Später ist der Titel der Website oben im Fenster ein Menü: Wechseln Sie
zu einer zuletzt geöffneten Website, öffnen Sie eine andere, legen Sie
eine neue an oder schließen Sie sie.

## Sich zurechtfinden

Die Tabs unter dem Titel enthalten alles zur geöffneten Website:

- **Pages**: die Seiten, Beiträge und geteilten Blöcke der Website, und
  der Editor für die, die Sie wählen. Siehe
  [Seiten und Blöcke](pages-and-blocks.md).
- **Media**, **Downloads** und **Static**: Bilder, Dateien zum
  Herunterladen und Dateien, die die Website so braucht, wie sie sind.
  Siehe [Bilder und Dateien](pictures-and-files.md).
- **Site**: Titel, Beschreibung, Sprachen, die Webadresse, Logo, Suche
  und Seitenbreite. Siehe [Einstellungen der Seite](site-settings.md).
- **Menu**: die Seitenleiste, und ein Menü von Hand, falls das
  automatische nicht passt. Siehe [Menüs](menus.md).
- **Theme**: Farben und Schriften. Siehe
  [Farben und Schriften](colors-and-fonts.md).
- **Publish**: die Website bauen und online stellen. Siehe
  [Veröffentlichen](publishing.md).
- **Versions**: die Geschichte der Website. Siehe
  [Versionen](versions.md).
- **App**: Einstellungen für q.e.d. selbst: seine eigenen Farben (alle
  Bildschirmfotos hier verwenden *catppuccin-mocha*), die Sprache des
  Handbuchs, Updates und die Einrichtung für
  [KI-Assistenten](ai-assistants.md).

Eine Podcast-Website hat ein paar mehr: **Podcast**, **Audio**,
**Transcripts** und **Contributors**. Siehe [Podcasts](podcasts.md).

::: image
image = "manual/app-tab.png"
alt = "Der Tab App von q.e.d.: ein Raster von Farb-Themes für die App, catppuccin-mocha ausgewählt, darunter die Version und eine Schaltfläche, um nach Updates zu suchen."
caption = "Der Tab **App**: die eigenen Farben von q.e.d., und Updates."
:::

## Das Ergebnis ansehen

:icon-eye: Die runde Schaltfläche mit dem Auge, oben rechts, baut die
Website und öffnet sie in Ihrem Browser, bei der Seite, die Sie gerade
bearbeiten. Online geht dabei nichts: Die Vorschau läuft nur auf Ihrem
Computer. Ändern Sie etwas, speichern Sie und drücken Sie sie erneut, um
die Änderung zu sehen.

## Speichern

:icon-device-floppy: Nichts ist gespeichert, bis Sie **Save** drücken. Ein Punkt
an einem Tab und die Worte *Unsaved changes* zeigen, dass etwas zu
speichern ist, und **Discard** verwirft es. Jedes Speichern bleibt in
der Geschichte der Website erhalten, Sie können also immer zurück; siehe
[Versionen](versions.md).

## Dieses Handbuch

:icon-book: **Manual**, neben der Vorschau-Schaltfläche, öffnet
dieses Handbuch in einem eigenen Fenster, für Ihre Version von q.e.d.,
ohne Internetverbindung. Manche Stellen in der App verlinken die Seite,
die sie erklärt.
