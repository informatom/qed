+++
title = "Icons"
description = "Die mehr als 6.000 Icons, die q.e.d. mitbringt, für Text und Menüs: eines finden, es schreiben, und eigene hinzufügen."
menu = "main"
weight = 25
summary = "Über 6.000 Icons für Text und Menüs, und Ihre eigenen."
+++

Icons stehen im Text und neben Seiten in Menüs. Sie werden in der Farbe
des Textes gezeichnet, so hoch wie die Buchstaben, und brauchen nichts
von anderen Websites. q.e.d. bringt die
[Tabler-Icons](https://tabler.io/icons) mit: mehr als 5.000 als Umriss
gezeichnete und 1.000 gefüllte, darunter Logos von Marken und Diensten,
etwa :icon-brand-mastodon: Mastodon.

## Ein Icon finden

Suchen Sie danach, was es zeigt. Die Suche kennt mehr als die Namen:
*envelope* findet :icon-mail: `mail`, *house* findet :icon-home: `home`.
Die Suchwörter sind englisch. Suchen können Sie an zwei Stellen in q.e.d.:

- unter **Icons in text** unter jedem Textfeld; ein Klick kopiert den
  Code, den Sie dann in den Text einfügen
- im Tab **Settings** einer Seite, bei **Icon**

[tabler.io/icons](https://tabler.io/icons) zeigt ebenfalls alle Icons
mit ihren Namen. Der Name eines gefüllten Icons endet auf `-filled`:
:icon-heart: `heart` und :icon-heart-filled: `heart-filled`.

## Im Text

Schreiben Sie den Namen des Icons zwischen Doppelpunkte und `icon-`:
`:icon-phone:` ergibt :icon-phone:. Das funktioniert in jedem Block mit
Text: einem Block Text, einer Spalte, einer FAQ-Antwort, der Fußzeile.
Eine Zeile wie

```text
:icon-phone: 01 234 56 78 · :icon-mail: hallo@example.com
```

erscheint als :icon-phone: 01 234 56 78 · :icon-mail: hallo@example.com.

Einen Namen, den q.e.d. nicht kennt, zeigt die Seite als Text, und
**Build** weist darauf hin, damit ein Tippfehler nicht unbemerkt bleibt.

## Neben einer Seite in Menüs

Der Tab **Settings** einer Seite hat **Icon**: Suchen Sie eines und
klicken Sie es an, dann steht es neben der Seite in den Menüs. **No
icon** nimmt es wieder weg.

## Eigene Icons

Für Logos und Bilder, die nicht dabei sind, etwa das Logo eines
Programms, über das Sie schreiben: Legen Sie eine Bilddatei (am besten
SVG) in den Ordner `icons` im Ordner `media` der Website. **Show in file
manager** im Tab Media öffnet ihn. `rails.svg` dort schreiben Sie
`:icon-rails:`. Eigene Icons behalten ihre eigenen Farben.

## Einige für den Anfang

| Icon | Schreiben | | Icon | Schreiben |
| --- | --- | --- | --- | --- |
| :icon-mail: | `:icon-mail:` | | :icon-phone: | `:icon-phone:` |
| :icon-map-pin: | `:icon-map-pin:` | | :icon-clock: | `:icon-clock:` |
| :icon-calendar: | `:icon-calendar:` | | :icon-home: | `:icon-home:` |
| :icon-info-circle: | `:icon-info-circle:` | | :icon-help-circle: | `:icon-help-circle:` |
| :icon-download: | `:icon-download:` | | :icon-link: | `:icon-link:` |
| :icon-rss: | `:icon-rss:` | | :icon-microphone: | `:icon-microphone:` |
| :icon-headphones: | `:icon-headphones:` | | :icon-book: | `:icon-book:` |
| :icon-heart-filled: | `:icon-heart-filled:` | | :icon-star-filled: | `:icon-star-filled:` |
| :icon-brand-mastodon: | `:icon-brand-mastodon:` | | :icon-brand-github: | `:icon-brand-github:` |

## Websites von vor Version 0.15

Bis Version 0.14 hatte q.e.d. 72 Icons von [Lucide](https://lucide.dev).
Ihre Namen funktionieren weiter: Eine Seite mit `:icon-house:` oder dem
Icon `house` zeigt jetzt :icon-home: `home`. Neue Icons wählen Sie mit
ihren Tabler-Namen.
