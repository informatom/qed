+++
title = "Farben und Schriften"
description = "Der Tab Theme: ein Theme wählen, entscheiden, welche seiner Farben wohin kommt, Schriften wählen, oder eigene Farben machen und unter einem Namen behalten; heller und dunkler Modus."
menu = "main"
weight = 36
summary = "Ein Theme wählen, seine Farben verteilen, Schriften wählen, eigene Farben machen."
+++

Wie die Website aussieht, stellen Sie im Tab **Theme** ein, für eine
helle und, wenn Sie möchten, eine dunkle Seite. Alles dort wird
gespeichert, sobald Sie es wählen; sehen Sie sich die Website erneut in
der Vorschau an (:icon-eye:). Rechts zeigt eine kleine Skizze einer
Seite das Ergebnis, während Sie wählen.

## Ein Theme

Unten im Tab stehen die Themes, unter **Light** und **Dark**, jedes mit
seinen Farben in einer Reihe. Klicken Sie eines an, und die ganze
Website übernimmt seine Farben. Jedes Theme hat vier Farben
(**primary**, **secondary**, **accent** und **neutral**) auf seinem
Hintergrund.

## Welche Farbe wohin kommt

Unter **Colors** entscheiden Sie für jeden Teil der Seite, welche der
Farben des Themes er bekommt:

- **Header**: **Plain** (der Hintergrund der Seite) oder eine farbige
  Leiste.
- **Links** und **Buttons**: eine der vier Farben.
- **Hero**, **Call to action**, **Boxed columns**, **Content
  background** und **Footer**: **plain**, **shaded** (eine Spur dunkler
  als die Seite) oder eine der vier Farben. Der Inhaltshintergrund nimmt
  nur einen hellen Hauch davon, damit Text lesbar bleibt.

Bei einem dunklen Theme ist die Fußzeile **shaded**, außer Sie wählen
eine andere Farbe: Die Akzentfarbe des Themes ist oft ein blasser Ton,
der ein helles Band unter einer dunklen Seite ergäbe. Bei einem hellen
Theme ist sie **accent**.

**Reset** setzt den Kopf und jede Farbe auf die Voreinstellung zurück.

## Heller und dunkler Modus

Viele Menschen stellen ihren Computer oder ihr Handy auf dunkle Farben,
zumindest am Abend. Schalten Sie oben im Tab **Light and dark mode**
ein, und die Website bekommt für sie ein zweites Aussehen: ein dunkles
Theme mit eigenen Farben. Besucher sehen dann die Seite, die zu ihrem
System passt. Drei kleine Schaltflächen neben der Suche im Kopf (eine
Sonne, ein Computer und ein Mond) lassen sie hell, die Einstellung
ihres Systems oder dunkel wählen; der Browser merkt sich die Wahl für
die ganze Website.

Beim Einschalten beginnt die dunkle Seite mit einem dunklen Theme, das
zum hellen passt (*catppuccin-mocha* zu *catppuccin-latte*, *dark* zu
*light*), und mit derselben Farbverteilung. **Light** und **Dark** neben
dem Schalter bestimmen, welche Seite der Rest des Tabs ändert: Theme,
Kopf, Farben und eigene Farben gibt es auf beiden Seiten getrennt.
Gespeicherte Farben lassen sich auf beiden Seiten verwenden. Die
Schriften sind für beide gleich.

Schalten Sie es wieder aus, werden die Einstellungen der dunklen Seite
entfernt; die helle Seite bleibt, wie sie war.

## Schriften

**Text** und **Headings** haben je **Change…**, das etwa 2.000 offene
Schriften durchsucht (darunter die Google Fonts). Wählen Sie eine, und
q.e.d. lädt sie einmal in die Website herunter; die Website liefert sie
dann selbst aus, sodass kein Browser eines Besuchers Google nach etwas
fragt. Überschriften verwenden die Textschrift, außer Sie wählen eine
andere; **Use system font** kehrt zur Schrift des Computers des
Besuchers zurück.

## Eigene Farben

Für eigene Farben, etwa die eines Logos, drücken Sie unter **Custom
colors** **Customize** neben dem Namen des Themes:

- **Start from**: das Theme, dessen Farben Sie ändern.
- **Background**, **Primary**, **Secondary**, **Accent**, **Neutral**:
  je eine Farbauswahl. Text auf jeder Farbe wird von selbst lesbar
  gemacht.
- **Corners**: **Sharp**, **Soft**, **Round**, oder wie das Theme sie
  hat.
- **Reset to …** übernimmt wieder alle Farben des Ausgangs-Themes.

Unter **Saved colors** geben Sie ihnen einen Namen und drücken **Save
these colors**. Dann können Sie andere Themes ausprobieren und mit einem
Klick zurückkommen.

**Ein Beispiel.** Der laufende Podcast
[3 Schweinehunde](https://3-schweinehun.de/) beginnt mit *light* und
ändert es: ein Mintgrün (`#48cfad`) als Primär- und Akzentfarbe, ein
helleres als Sekundärfarbe, Dunkelgrau als Neutral und weiche Ecken,
gespeichert als „3 Schweinehunde“. Die Kopfleiste ist primary, die
Fußzeile shaded, und Text und Überschriften verwenden die Schrift
*Ubuntu*.

::: image
image = "manual/theme-tab.png"
alt = "Der Tab Theme von q.e.d. für die Website 3 Schweinehunde: die Farbwahl für Kopf, Links, Schaltflächen und die anderen Teile der Seite, die Schrift Ubuntu, die eigenen Mintfarben gespeichert als 3 Schweinehunde, darunter das Raster der Themes, und rechts eine Skizze einer Seite, die zeigt, wohin jede Farbe kommt."
caption = "Der Tab **Theme** von 3 Schweinehunde: rechts, wohin jede Farbe kommt."
:::

::: faq
[[items]]
question = "Ich habe ein Theme gewählt, aber die Website sieht gleich aus."
answer = "Die Vorschau zeigt die Website, wie sie gebaut wurde. Drücken Sie erneut :icon-eye:, um sie mit den neuen Farben zu bauen."

[[items]]
question = "Können Besucher zwischen hell und dunkel wechseln?"
answer = "Ja, mit **Light and dark mode** oben im Tab Theme: ein zweites, dunkles Theme für Besucher, deren System auf dunkel steht, und ein Umschalter im Kopf. Ohne es hat die Website das eine Theme, das Sie wählen."

[[items]]
question = "Was sehen Besucher ohne JavaScript?"
answer = "Die Seite, die zur Einstellung ihres Systems passt. Nur der Umschalter braucht JavaScript; ohne es wird er nicht gezeigt."
:::
