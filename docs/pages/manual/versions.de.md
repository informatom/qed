+++
title = "Versionen"
description = "Jedes Speichern bleibt erhalten: die offene Version, Versionen mit einer Notiz, zurück zu einem früheren Stand einer Seite oder der ganzen Website, und gelöschte Seiten zurückholen."
menu = "main"
weight = 40
summary = "Jedes Speichern bleibt erhalten; zurück zu einem früheren Stand einer Seite oder der ganzen Website."
+++

q.e.d. bewahrt die Geschichte Ihrer Website auf, jedes Speichern, im
Ordner der Website. Nichts, was Sie gespeichert haben, geht verloren,
und alles lässt sich rückgängig machen.

## Die offene Version

:icon-history: Was Sie speichern, kommt in die **offene Version**: eine
Version, die mit jedem Speichern wächst, während Sie arbeiten. Der Tab
**Versions** zeigt sie ganz oben, als *open* markiert, mit ihrem Beginn
und Ihrem letzten Speichern.

Sie wird geschlossen und zu einer eigenen Version, wenn:

- Sie ihr eine Notiz geben und **Close version** klicken: Sagen Sie in
  ein paar Worten, was Sie getan haben, etwa *Neue Öffnungszeiten*;
- Sie die Website veröffentlichen (Publish fragt, was neu ist, und das
  wird die Notiz);
- Sie sie sichern oder Änderungen von einem anderen Computer holen (Push
  oder Pull, siehe [Sicherungen](backups.md));
- Sie nach einer Pause zurückkommen: nach einer Stunde ohne Speichern,
  oder am nächsten Tag, fragt q.e.d. nach einer Notiz für das, was Sie
  vorher getan haben. **Use suggested note** benennt sie nach ihren
  Zeiten, etwa *Saves on 27 Oct, 13:50–21:12*.

So wird jede Arbeitssitzung zu einer Version, zu der Sie zurück können.

::: image
image = "manual/versions-tab.png"
alt = "Der Tab Versions einer Website: links ein Feld für eine Notiz und die Schaltfläche Save version, sowie Push und Pull; rechts die Liste der Versionen mit ihren Notizen, etwa Saves on 27 Oct, 21:12–21:25 und Before moving /datenschutz, jede mit einer Schaltfläche Go back, und darunter Deleted pages mit einer Schaltfläche Restore."
caption = "Der Tab **Versions**: Versionen mit ihren Notizen, **Go back** für die ganze Website, und **Deleted pages**."
:::

## Zurückgehen

:::: columns
::: column
title = "Eine Seite"
boxed = true
:::

Öffnen Sie die Seite und ihren Tab **Versions**. Wählen Sie eine
Version, um zu sehen, was sie geändert hat, und :icon-history: **Restore
this version**, um sie in den Editor zu holen. Gespeichert wird nichts,
bis Sie **Save** klicken, Sie können also erst schauen.

::: column
title = "Die ganze Website"
boxed = true
:::

:icon-undo: **Go back** neben einer Version im Tab **Versions** stellt
jede Seite, jedes Bild und jede Datei so wieder her, wie sie damals
waren. Der jetzige Stand wird vorher als Version gespeichert, Sie
kommen also auf demselben Weg zu ihm zurück.
::::

## Gelöschte Seiten

Unter **Deleted pages** listet der Tab **Versions** gelöschte Seiten,
mit der letzten Version, die sie noch hatte. :icon-undo: **Restore**
holt eine so zurück, wie sie vor dem Löschen war.

::: call-to-action
headline = "Eine Kopie auf einem anderen Computer"
text = "Die Geschichte liegt im Ordner der Website. Geht der Computer kaputt, ist sie mit ihm weg. Eine Sicherung auf einem anderen Computer bewahrt jede Version."
button_label = "Sicherungen"
button_link = "backups.md"
:::
