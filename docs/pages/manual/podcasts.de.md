+++
title = "Podcasts"
description = "Eine Website zum Podcast machen: seine Einstellungen, die Menschen, Audiodateien und Transkripte, neue Episoden mit Kapiteln, das Episoden-Layout und die Feeds für Podcast-Apps."
menu = "main"
weight = 70
summary = "Podcast-Einstellungen, Mitwirkende, Episoden, Audio und Feeds."
+++

Jede q.e.d.-Website kann ein Podcast sein. Die Episoden sind Seiten der
Website, mit einem Player, den Menschen, die sprechen, Shownotes und
Kapiteln, und q.e.d. schreibt die Feeds, die Podcast-Apps abonnieren.
Alles bleibt auf Ihrem eigenen Webspace: das Audio, der Player, die
Feeds.

Das Beispiel auf dieser Seite ist
[3 Schweinehunde](https://3-schweinehun.de/), ein laufender Podcast mit
vier Gastgebern und mehr als 50 Episoden.

## Einschalten

Schalten Sie im Tab **Podcast** **This site is a podcast** ein, füllen
Sie die Einstellungen aus und drücken Sie **Save**. Drei weitere Tabs
erscheinen: **Audio**, **Transcripts** und **Contributors**.

:::: columns
::: column
title = "Über den Podcast"
boxed = true
:::

:icon-podcast: **Name in podcast apps** (ohne Eintrag der Titel der
Website), **Author** (unter dem Namen gezeigt) und **Description**.

3 Schweinehunde: „Birgit, Dominic, Stefan und Steve“ als Autor.

::: column
title = "Podcast-Verzeichnisse"
boxed = true
:::

:icon-search: **Owner** und **Owner email**, wohin Verzeichnisse wie
Apple Podcasts Bestätigungen schicken; **Categories** aus Apples Liste
(die erste zählt am meisten); **Type**, **Explicit content** und
**Copyright or license**.

3 Schweinehunde: *Sports > Running*, CC BY 4.0.

::: column
title = "Cover"
boxed = true
:::

:icon-image: Ein quadratisches Bild, 1400 bis 3000 Pixel, JPEG oder PNG.
Podcast-Apps zeigen es überall. q.e.d. prüft es nach Apples Regeln.
::::

::: image
image = "manual/podcast-tab.png"
alt = "Der Tab Podcast von q.e.d. für 3 Schweinehunde: der Schalter This site is a podcast, Name und Autor, die Beschreibung, Owner und Owner email, die Kategorie Running, der Typ Episodic, die Lizenz CC BY 4.0 und die Podcast-GUID."
caption = "Der Tab **Podcast** von 3 Schweinehunde."
:::

Füllen Sie auch **Copyright or license** aus: Es sagt den Hörerinnen und
Hörern, was sie mit den Episoden tun dürfen (siehe
[Lizenz und Dank](license-and-credits.md)).

**Team** listet die Menschen, die den Podcast machen (siehe unten); neue
Episoden beginnen mit ihnen. **Support links** (Liberapay, Patreon, …)
erscheinen in Podcast-Apps als „Unterstützen“.

## Audiodateien

Unter **Audio files** im Tab Podcast:

- **Formats**: MP3 reicht. Weitere Formate (M4A, Ogg, Opus, FLAC)
  bekommen je einen eigenen Feed, und jede Episode braucht dann eine
  Datei in jedem. Bei mehr als einem wählen Sie das **Main format**:
  Sein Feed ist der für Apple und die Verzeichnisse, daher sollte es MP3
  oder M4A sein.
- Wo die Dateien liegen:
  - **In the site, not in versions**: in der Website, aber nicht in
    ihrer Geschichte. Die Versionen bleiben klein; das Audio sichern Sie
    selbst.
  - **In the site and in versions**: auch in jeder Version. Eine
    Sicherung enthält alles, aber die Geschichte wächst mit jeder
    Episode für immer.
  - **Elsewhere**: bei einem Podcast-Hoster; jede Episode nennt die
    Adresse ihrer Datei.

Liegen die Dateien in der Website, kopiert **Import audio…** im Tab
**Audio** sie hinein, etwa `3sh051_trilogie.mp3`.

Die Dateien einer Episode in verschiedenen Formaten müssen dieselbe
Aufnahme sein, gleich geschnitten: Die Kapitel und die Länge gelten für
alle, und wer das Format wechselt, landete sonst an der falschen
Stelle.

## Mitwirkende

Alle, die in einer Episode sprechen, tragen Sie einmal im Tab
**Contributors** ein: **New contributor**, mit **Name**, einem **Photo**
(quadratisch geht am besten), **Links** (Website, Mastodon, …) und
**About them**. Jede und jeder bekommt eine eigene Seite auf der
Website, mit ihren Episoden.

3 Schweinehunde hat 34: die vier Gastgeber und die Gäste einzelner
Episoden.

## Eine neue Episode

:icon-mic: **New episode** im Tab Pages fragt nach Titel und Nummer; sie
zählt von der letzten Episode weiter. Die Episode ist auf heute datiert
und kommt unter `/episodes`.

Der Tab **Settings** der Episode hat, neben Titel, Datum und
Beschreibung:

- **Episode**: **Season**, **Number**, **Type** (**Episode**,
  **Trailer**, **Bonus**), **Explicit**, und ein **Episode cover**
  (ohne eines das Cover des Podcasts).
- **Audio**: **Choose…** die Datei für jedes Format aus dem Tab Audio;
  q.e.d. liest die **Length** daraus. Ein **Transcript** kommt aus dem
  Tab Transcripts (eine WebVTT-Datei, `.vtt`).
- **People**: wer dabei ist, jede Person in einer Rolle (**Host**,
  **Guest**, …). Wer zwei Rollen hat, bekommt zwei Zeilen.
- **Chapters**: je eine Startzeit (`01:02:03` oder `02:03`) und ein
  Titel, der Reihe nach; ein Link ist freiwillig. Episode 51 von 3
  Schweinehunde hat eine Einleitung und ein Kapitel für jedes
  Läuferpaar.

::: image
image = "manual/episode-settings.png"
alt = "Die Episoden-Einstellungen von Episode 51 von 3 Schweinehunde: Nummer 51, Typ Episode, die MP3-Datei 3sh051_trilogie.mp3 mit 74 MB und einer Länge von 1:28:05, vier Personen (drei Gastgeber und Sonja Waldgruber als Gast) und vier Kapitel von Einleitung bis Sonja und Stefan."
caption = "Der Episoden-Teil des Tabs **Settings**, für Episode 51 von 3 Schweinehunde."
:::

Die eigenen Blöcke der Episode sind ihre **Shownotes**: Text, Links,
Bilder, wie auf jeder Seite. Ein paar Sätze, worum es in der Episode
geht, dann die Links zu dem, was erwähnt wurde, sind ein guter Anfang;
auch Podcast-Apps zeigen die Shownotes.

::: call-to-action
headline = "Publish wartet auf das Audio"
text = "Jeder Feed braucht jede Episode. Solange einer Episode, die kein Entwurf ist, ihr Audio fehlt (in einem der Formate), sagt Publish welche und wartet. Machen Sie eine Episode zum Entwurf, solange sie nicht fertig ist."
:::

## Die Episodenseite

Jede Episodenseite zeigt unter ihrem Titel: den **Player**, die
**People**, die **Shownotes** und die **Chapters**. Der Player ist der
Podlove Web Player, mit Kapiteln, Transkript und einem Tab zum Teilen;
ohne JavaScript der eigene Player des Browsers und ein Download-Link.

Um diese Reihenfolge für alle Episoden zu ändern oder jeder Episode
einen Block hinzuzufügen (etwa einen Subscribe-Button), öffnen Sie
**Episode layout** unter **Templates** in der Seitenliste.
**Transcript** zeigt das Transkript als Text, zugeklappt, bis ein
Besucher es öffnet.

## Blöcke für Podcasts

Jede Seite einer Podcast-Website kann verwenden:

- **Latest episode**: die neueste Episode mit ihrem Player.
- **Subscribe**: den Podlove Subscribe Button, der die Feeds des
  Podcasts der Podcast-App des Besuchers anbietet.
- **Contributors**: die Menschen, mit Fotos, jede Person verlinkt auf
  ihre Seite.

Die Startseite von 3 Schweinehunde hat Latest episode, Subscribe und
eine Post list der anderen Episoden.

## Feeds und Podcast-Apps

Die Website braucht für ihre Feeds ihre **Web address** (Tab Site). Sie
liegen unter `/episodes.mp3.rss` (und je einem pro weiterem Format),
etwa `https://3-schweinehun.de/episodes.mp3.rss`. Reichen Sie diese
Adresse einmal bei Apple Podcasts, Spotify und den anderen
Verzeichnissen ein; danach holen sie jede neue Episode ab, wenn Sie
veröffentlichen.

Der Podcast ist in der Hauptsprache der Website; seine Feeds und
Episoden gibt es nur dort.

::: faq
[[items]]
question = "Kann ich einer bestehenden Website einen Podcast hinzufügen?"
answer = "Ja. Schalten Sie ihn im Tab Podcast ein; die Seiten bleiben, wie sie sind, und die Episoden kommen unter /episodes."

[[items]]
question = "Was passiert mit den Episoden, wenn ich den Podcast ausschalte?"
answer = "Die Podcast-Einstellungen verschwinden beim Speichern; die Episodenseiten bleiben, als gewöhnliche Seiten."

[[items]]
question = "Mein Podcast ist gerade woanders. Kann ich ihn herüberholen?"
answer = "Siehe [Eine Website umziehen](moving-a-site.md)."
:::
