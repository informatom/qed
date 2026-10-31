+++
title = "Einstellungen der Seite"
description = "Der Tab Site: Titel, Sprache, Beschreibung und Webadresse, weitere Sprachen, Logo und Website-Icon, Links zu anderen Websites und Ihre Profile dort, die Suche und die Seitenbreite."
menu = "main"
weight = 32
summary = "Titel, Sprachen, Webadresse, Logo, Icon, Suche und Seitenbreite."
+++

Der Tab **Site** enthält, was zur ganzen Website gehört. Ändern Sie, was
Sie möchten, und drücken Sie dann **Save**; **Discard** vergisst die
Änderungen.

## Allgemein

- **Title**: der Name der Website, im Kopf jeder Seite und im
  Browser-Tab.
- **Language**: ein kurzes Kürzel wie `en`, `de` oder `de-AT`. Es sagt
  Browsern und Suchmaschinen die Sprache und bestimmt, wie Daten
  geschrieben werden: Mit `de-AT` heißt der Januar *Jänner*.
- **Description**: ein, zwei Sätze über die Website, von Suchmaschinen
  gezeigt.
- **Web address**: wo die Website online sein wird, etwa
  `https://example.com/`. Feeds, die Sitemap und Links zum Teilen
  brauchen sie; tragen Sie sie vor dem Veröffentlichen ein.

## Sprachen

Eine Website kann mehr als eine Sprache haben: zum Beispiel Deutsch
unter `/` und Englisch unter `/en/`. Die Sprache unter Allgemein ist die
Hauptsprache; weitere fügen Sie hier mit ihrem Kürzel hinzu (**Add a
language**), etwa `en`, `fr` oder `pt-br`. Jede kann einen eigenen
**Titel** und eine eigene **Beschreibung** der Website haben; leer
bleiben die der Hauptsprache.

Übersetzen Sie dann Seite für Seite: Öffnen Sie eine Seite und drücken
Sie **Translate** neben dem Kürzel der Sprache darüber (siehe
[Seiten und Blöcke](pages-and-blocks.md)). Eine Seite ohne Übersetzung
gibt es in dieser Sprache einfach nicht. Menüs, Beitragslisten und Feeds
jeder Sprache zeigen nur ihre eigenen Seiten, und ein Umschalter im Kopf
(*DE EN*) führt zur selben Seite in der anderen Sprache, oder zur
Startseite dieser Sprache, wo es keine gibt.

Eine Sprache, die noch Übersetzungen hat, lässt sich nicht entfernen;
löschen Sie diese zuerst.

## Logo

Wählen Sie ein Bild aus Media; es steht im Kopf jeder Seite und, ohne
Website-Icon, als Icon im Browser-Tab. Eine SVG-Datei bleibt in jeder
Größe scharf.

- **Size**: **Small**, **Medium**, **Large** oder **Original** für die
  eigene Größe des Bildes (bis zu 112 Pixel hoch), was kleine
  Bitmap-Logos scharf hält.
- **Show the site title next to the logo**: Schalten Sie es aus, wenn
  das Logo den Namen schon zeigt.

## Website-Icon

Das kleine Bild im Browser-Tab, in Lesezeichen und auf dem Startbildschirm
eines Handys, wenn jemand die Website dort ablegt. Ein breites Logo gibt
ein schlechtes Icon, daher kann es ein eigenes Bild sein: Fügen Sie es in
Media hinzu und wählen Sie es hier. q.e.d. verwendet es so, wie es ist,
daher muss es eines davon sein:

- ein **SVG**, in jeder Größe scharf;
- ein quadratisches **PNG**, mindestens 180 × 180 Pixel, damit es auf
  Handys scharf ist;
- eine **ICO**-Datei, das klassische `favicon.ico`.

Andere Bilder, und ein PNG, das nicht quadratisch ist, lehnt das
Speichern ab. Ohne Website-Icon zeigt der Browser-Tab das Logo.

## Links

**Mark links to other sites** gibt Links im Text, die zu anderen
Websites führen, einen kleinen Pfeil, damit Besucher wissen, dass sie
die Seite verlassen. Menüs und Schaltflächen bleiben, wie sie sind.

**Your profiles elsewhere** listet Ihre Profile auf anderen Websites,
etwa `https://mastodon.social/@name`: **Add a profile**, Adresse
einfügen. Jede Seite nennt sie dann unsichtbar als Ihre (`rel="me"`).
Mastodon zeigt Ihre Website damit als bestätigt, mit grünem Haken, in
Ihrem Profil:

1. Fügen Sie das Mastodon-Profil hier hinzu, speichern und
   veröffentlichen Sie.
2. Bearbeiten Sie auf Mastodon Ihr Profil und tragen Sie die Adresse
   Ihrer Website in eines der Zusatzfelder ein.
3. Mastodon sieht sich diese Seite an, findet das dort genannte Profil
   und markiert das Feld als bestätigt. Das kann ein paar Minuten
   dauern.

Um auf der Website auch einen Link *Folgen Sie mir* zu zeigen, setzen
Sie einen gewöhnlichen Link in die Fußzeile oder auf eine Seite.

## Suche

**Site search** setzt eine Suchschaltfläche in den Kopf (auch Strg+K).
Sie findet Wörter in allen Seiten, Beiträgen und Episoden. Die Suche
läuft im Browser des Besuchers, aus Dateien auf Ihrer eigenen Website;
nichts geht an andere Dienste. Für neue Websites ist sie eingeschaltet.

## Layout

**Page width**: **Normal** passt zu Text, **Wide** gibt Spalten Platz,
**Full** nutzt das ganze Fenster.

::: call-to-action
headline = "Farben, Schriften und Menüs"
text = "Wie die Website aussieht, steht im Tab Theme, das Menü im Tab Menu."
button_label = "Farben und Schriften"
button_link = "colors-and-fonts.md"
:::
