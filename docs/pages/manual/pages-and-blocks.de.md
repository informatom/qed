+++
title = "Seiten und Blöcke"
description = "Seiten und Blogbeiträge anlegen, aus Blöcken bauen, Blöcke in Spalten und Abschnitten anordnen, Blöcke zwischen Seiten teilen, Seiten verlinken und übersetzen."
menu = "main"
weight = 20
summary = "Seiten und Beiträge schreiben, aus Blöcken, Spalten und Abschnitten bauen, verlinken."
+++

## Seiten und Beiträge

Der Tab **Pages** zeigt die Seiten der Website als Baum: Eine Seite
unter einer anderen hat ihre Adresse darunter, etwa `/about/team` unter
`/about`. Die Startseite ist `/`. Ziehen Sie eine Seite in der Liste, um
sie an anderer Stelle einzureihen oder unter eine andere Seite zu
stellen.

:icon-file-plus: **New page** fragt nach einem Titel und wohin die Seite
kommt (**Below**: oberste Ebene oder eine andere Seite). Schalten Sie
**Blog post** ein und wählen Sie ein Datum, um einen Beitrag anzulegen;
die Uhrzeit ist mit der aktuellen ausgefüllt. Beiträge werden nach Datum
und Uhrzeit gelistet, die neuesten zuerst, auf der Seite, unter der sie
stehen (meist der Blog), und sie bekommen einen Feed. Die Uhrzeit hält
Beiträge eines Tages in Feed-Readern in der richtigen Reihenfolge; sie
gilt in der Zeitzone Ihres Computers, und Sie können sie im Tab Settings
der Seite ändern oder leer lassen. Ein
Blog mit vielen Beiträgen kann sie mit einer Seite pro Tag, Monat oder
Jahr darunter gruppieren, benannt nach dem Datum (`/blog/2026-10-21`,
`/blog/2026`) und mit einer eigenen Post list: Die Beiträge darunter
kommen trotzdem in den einen Feed des Blogs. Der Dialog zeigt die
Adresse, die die Seite bekommt, aus dem Titel gemacht; Sie können sie
später ändern.

::: image
image = "manual/new-page.png"
alt = "Der Dialog New page: der Titel My new blog post, unter dem Blog, Blog post eingeschaltet mit dem heutigen Datum, die Adresse /blog/2026-10-29-my-new-blog-post, und die Schaltflächen Cancel und Create page."
caption = "Ein neuer Blogbeitrag: unter dem Blog, mit einem Datum."
:::

Wählen Sie eine Seite, um sie zu bearbeiten. Der Editor hat drei Tabs:

- **Content**: die Blöcke der Seite (unten).
- **Settings**: Titel, Beschreibung, Menü, Reihenfolge und mehr (weiter
  unten).
- **Versions**: frühere Stände dieser Seite (siehe
  [Versionen](versions.md)).

::: image
image = "manual/pages-tab.png"
alt = "Der Tab Pages von q.e.d.: links die Liste der Seiten als Baum mit ihren Adressen und Menü-Abzeichen, rechts die Startseite im Editor, mit ihren Blöcken Image und Hero und den Tabs Content, Versions und Settings."
caption = "Der Tab **Pages**: links die Seiten, rechts die gewählte im Editor."
:::

Die Schaltflächen neben den Tabs verschieben oder benennen die Seite um
(:icon-folder-down:; sie behält ihre alte Adresse, gelistet unter **Old
addresses** in ihrem Tab Settings, und Links auf sie werden angepasst)
und löschen sie (:icon-trash:). Die Startseite lässt sich weder
verschieben noch löschen.

## Blöcke

Eine Seite besteht aus Blöcken, einer unter dem anderen. Die Zeile ganz
unten (**Add:**) fügt einen hinzu; die Pfeile schieben einen Block nach
oben oder unten, der Griff links zieht ihn, und :icon-trash: entfernt
ihn.

- **Text**: Markdown, eine einfache Art, formatierten Text zu schreiben
  (siehe unten).
- **Hero**: ein großer Kopfbereich mit Überschrift, Text und einer
  Schaltfläche, auf einem Bild oder einer Farbe.
- **Image**: ein Bild mit Alternativtext und Bildunterschrift.
- **Text with image**: Text neben einem Bild, das Bild links oder
  rechts.
- **Gallery**: mehrere Bilder in einem Raster; jedes öffnet sich groß
  auf einen Klick.
- **Download**: eine Datei zum Herunterladen, mit Typ und Größe.
- **FAQ**: Fragen, die sich öffnen und ihre Antworten zeigen.
- **Columns**: zwei bis vier Spalten nebeneinander, jede mit eigenen
  Blöcken; auf Handys stehen sie untereinander.
- **Section**: ein Band mit Titel, Hintergrundfarbe oder -bild, das
  eigene Blöcke enthält, Spalten eingeschlossen.
- **Call to action**: ein hervorgehobener Kasten mit Überschrift, Text
  und einer Schaltfläche.
- **Post list** und **Page list**: die Beiträge oder Seiten unter einer
  Seite, als Liste, als Karten oder als Kacheln.
- **Video**: ein Video von YouTube, Vimeo oder PeerTube, als sein Bild
  gezeigt und verlinkt, oder eine Videodatei aus Downloads.
- **Shared block**: zeigt einen geteilten Block (unten).

Podcast-Websites haben dazu **Latest episode**, **Subscribe** und
**Contributors**.

Jedes Bild braucht einen **Alternativtext**: eine kurze Beschreibung für
Menschen, die es nicht sehen können, und für Suchmaschinen.

## Text schreiben

Textblöcke verwenden Markdown. Der Texteditor zeigt die Formatierung
schon beim Tippen, und seine Werkzeugleiste fügt sie für Sie ein:

- `**fett**` für **fett**, `*kursiv*` für *kursiv*
- `## Überschrift` in einer eigenen Zeile für eine Überschrift
- Zeilen `- Punkt` für eine Liste, `1. Punkt` für eine nummerierte
- `[der Text](/about)` für einen Link (mehr unten)
- `![was es zeigt](/media/foto.jpg)` für ein Bild aus Media

Vier HTML-Tags funktionieren im Text, für das, was Markdown nicht
schreiben kann:

- `<kbd>Strg</kbd>+<kbd>S</kbd>` für Tasten: <kbd>Strg</kbd>+<kbd>S</kbd>
- `m<sup>2</sup>` und `CO<sub>2</sub>`: m<sup>2</sup>, CO<sub>2</sub>
- `<br>` für eine neue Zeile, wo Markdown keine hat, etwa in einer
  Tabellenzelle

Die vier Schaltflächen nach den beiden Code-Schaltflächen (Code in einer
Zeile und ein Codeblock) in der Werkzeugleiste setzen sie für Sie ein,
um das, was Sie markiert haben. Jedes andere HTML erscheint so, wie es
geschrieben ist, als Text.

`:icon-phone:` setzt ein Icon in den Text, so hoch wie die Buchstaben
(alle stehen unter [Icons](icons.md)). Formeln (`$a^2 + b^2 = c^2$`) und
ein paar Dinge mehr funktionieren auch; der englische Blog zeigt sie.

## Spalten und Abschnitte

**Columns** stellen Blöcke nebeneinander. Jede Spalte hat einen
freiwilligen Titel, eine Breite (**2×** neben **1×** sind zwei Drittel)
und **Box**, die sie auf einem eigenen Hintergrund zeichnet. Manche
Blöcke brauchen die volle Breite und können nicht in eine Spalte: Hero,
Gallery, Columns und Section.

Eine **Section** fasst Blöcke auf einem Band zusammen: ein Titel
darüber, ein Hintergrund (keiner, ein Kasten, eine Schattierung, eine
der Farben des Themes oder ein Bild) und normaler oder großer Abstand.
Drei Spalten auf einem farbigen Band sind eine Section mit einem Block
Columns darin.

## Geteilte Blöcke

Manche Blöcke gehören auf mehrere Seiten: Öffnungszeiten, ein
Kontaktkasten. Klicken Sie :icon-components: **Make shared** an einem
Block und geben Sie ihm einen Namen: Er wandert in einen geteilten
Block, und ein **Shared block** tritt an seine Stelle. Fügen Sie ihn mit
**Add: Shared block** auf anderen Seiten ein. Bearbeiten Sie ihn einmal,
unter **Shared blocks** in der Seitenliste, und jede Seite zeigt die
Änderung.

Der **Footer**, unter **Templates** in der Seitenliste, enthält Blöcke,
die unten auf jeder Seite stehen.

## Links

Verlinken Sie eine andere Seite der Website mit ihrer Adresse, beginnend
mit einem Schrägstrich: `[Über uns](/about)`. Links zu anderen Websites
schreiben Sie vollständig: `[Podlove](https://podlove.org)`. Dateien in
Downloads verlinken Sie als `/assets/speisekarte.pdf`.

Beim Bauen oder in der Vorschau listet q.e.d. Links zu Seiten und
Dateien auf, die es nicht gibt, damit ein Tippfehler nicht unbemerkt
bleibt. Verschieben Sie eine Seite, werden die Links auf sie angepasst.

## Einstellungen einer Seite

Der Tab **Settings** einer Seite:

- **Title**, und eine **Description** für Suchmaschinen und
  Link-Vorschauen.
- **Share image**: das Bild, das gezeigt wird, wenn die Seite geteilt
  wird; ohne eines ihr erstes Bild.
- **Menu**: das Hauptmenü, das Fußzeilenmenü oder keines, und die
  **Order** unter ihren Nachbarn (siehe [Menüs](menus.md)).
- **Draft**: aus der Website ausgelassen, bis Sie es ausschalten.
- **Hide title**: für Seiten, deren erster Block ihn schon sagt.
- **Back and next links on the pages below**: Die Seiten unter dieser
  bekommen unten Links zur Seite davor und danach, für ein Handbuch oder
  einen Kurs, den man der Reihe nach liest. Beiträge bekommen stattdessen
  **Older** und **Newer**, wie in einem Blog. Dieses Handbuch und seine
  Seiten funktionieren so.
- **Icon**: neben der Seite in Menüs gezeigt.
- Für Beiträge: **Date**, **Authors**, **Tags** und eine **Summary** für
  Beitragslisten.

Einstellungen und Inhalt werden getrennt gespeichert, jedes mit seinem
eigenen **Save**.

## Übersetzungen

Auf einer Website mit mehr als einer Sprache (siehe
[Einstellungen der Seite](site-settings.md)) steht über einer geöffneten
Seite eine Zeile mit Sprachkürzeln. **Translate** neben einem Kürzel
beginnt die Übersetzung der Seite: eine Kopie der Seite, noch in der
Hauptsprache und als **Draft** gespeichert, damit sie nicht halb fertig
veröffentlicht wird. Übersetzen Sie Titel und Blöcke, schalten Sie
**Draft** in ihren Einstellungen aus, und sie ist da, unter derselben
Adresse unter dem Kürzel der Sprache (`/en/about` für `/about`).

In der Seitenliste stehen die Übersetzungen einer Seite als kleine
Kürzel in ihrer Zeile (*EN*); ein Klick öffnet eine. Eine Übersetzung
behält die Adresse ihrer Seite: Verschieben oder Umbenennen der Seite
nimmt ihre Übersetzungen mit. Eine Seite mit Übersetzungen lässt sich
erst nach ihnen löschen.

Ein Link auf eine andere Seite über ihre Datei (`[Über uns](about.md)`)
führt zur Übersetzung in der Sprache der Seite, wenn es eine gibt; eine
ausgeschriebene Adresse (`/about`, `/en/about`) führt genau dorthin.

## Fragen

::: faq
[[items]]
question = "Warum erscheint meine Seite nicht im Menü?"
answer = "Eine Seite steht nur in einem Menü, wenn ihre Einstellungen das sagen: **Menu** auf das Haupt- oder Fußzeilenmenü gestellt. Entwürfe (Draft) fehlen in der Website ganz."

[[items]]
question = "Ich habe eine Seite versehentlich gelöscht. Bekomme ich sie zurück?"
answer = "Ja. Der Tab **Versions** listet **Deleted pages** mit der letzten Version, die sie noch hatte; holen Sie sie von dort zurück. Siehe [Versionen](versions.md)."

[[items]]
question = "Jemand hat die Seite geändert, während ich sie bearbeitet habe. Was nun?"
answer = "q.e.d. bemerkt es beim Speichern und fragt: **Keep mine** speichert Ihre Fassung über die anderen Änderungen, **Take the file's** zeigt die Seite so, wie die Datei sie jetzt hat, **Cancel** speichert nichts. Das passiert, wenn die Seite in einem anderen Programm, von Hand oder auf einem anderen Computer geändert und hierher geholt wurde."

[[items]]
question = "Wie stelle ich zwei Dinge nebeneinander?"
answer = "Mit einem Block **Columns**: Jede Spalte enthält eigene Blöcke. Auf Handys stehen die Spalten untereinander."
:::
