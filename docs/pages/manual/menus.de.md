+++
title = "Menüs"
description = "Das Hauptmenü und das Fußzeilenmenü: von selbst aus den Seiten gemacht, oder von Hand mit Gruppen und Links angeordnet; oben in der Leiste oder in einer Seitenleiste."
menu = "main"
weight = 34
summary = "Haupt- und Fußzeilenmenü, von selbst oder von Hand."
+++

Eine Website hat zwei Menüs: das **Hauptmenü** oben auf jeder Seite und
das **Fußzeilenmenü** unten.

## Aus den Seiten gemacht

Meist ist nichts zu tun. Der Tab **Settings** jeder Seite sagt, wohin
sie gehört:

- **Menu**: **Main menu**, **Footer menu** oder **Not in a menu**.
- **Order**: eine Zahl; Seiten mit kleineren Zahlen kommen zuerst.
- **Icon**: neben der Seite im Menü gezeigt (siehe [Icons](icons.md)).

Seiten unter einer anderen Seite im Menü (die Beiträge eines Blogs, die
Seiten unter *Über uns*) gehen mit ihr: In der oberen Leiste wird sie zu
einem Aufklappmenü mit der Seite selbst zuerst und den anderen darunter;
ein langes lässt sich scrollen. In der Seitenleiste klappen sie zu
(siehe unten).

Neue Seiten stehen in keinem Menü, bis Sie eines wählen. Das
Fußzeilenmenü entsteht immer so; Impressum und Datenschutz gehören
dorthin.

Auf einer Website mit mehreren Sprachen hat jede Sprache ihre eigenen
Menüs, aus ihren eigenen Seiten.

## In einer Seitenleiste

Gibt es mehr Seiten, als in die obere Leiste passen, schalten Sie im Tab
**Menu** **Show the main menu in a sidebar** ein. Das Menü läuft dann an
der linken Seite jeder Seite entlang; auf Handys wird es zu einem Kasten
*Menu* über dem Inhalt. Es wird sofort gespeichert.

Eine Seite mit Seiten darunter klappt diese in der Seitenleiste weg: Ein
Klick auf ihren Namen öffnet die Seite, ein Klick auf den Pfeil daneben
zeigt oder verbirgt die Seiten darunter. Nur der Teil des Menüs mit der
Seite, auf der ein Besucher gerade ist, ist anfangs offen.

## Von Hand angeordnet

Um Seiten zu gruppieren, ihnen im Menü andere Titel zu geben oder Links
zu anderen Websites hinzuzufügen, drücken Sie im Tab **Menu** **Arrange
the menu by hand**. q.e.d. beginnt mit dem Menü, wie es ist, und Sie
ändern es:

:::: columns
::: column
title = "Page"
boxed = true
:::

:icon-file-text: Eine Seite der Website, mit ihrem Titel oder einem
eigenen für das Menü.

::: column
title = "Link"
boxed = true
:::

:icon-link: Ein Titel und eine Adresse: eine andere Website
(`https://…`) oder eine Adresse auf Ihrer Website (`/blog`).

::: column
title = "Group"
boxed = true
:::

:icon-list: Ein Titel mit Seiten und Links darunter: ein Aufklappmenü in
der oberen Leiste. In der Seitenleiste klappt sie mit einem Klick auf
und zu; nur die Gruppe der Seite, auf der ein Besucher ist, ist anfangs
offen. Eine **Line** in einer Gruppe teilt sie.
::::

Die Pfeile schieben einen Eintrag nach oben und unten, :icon-trash:
entfernt ihn. **Save menu** speichert es; **Back to automatic** kehrt
zum Menü aus den Einstellungen der Seiten zurück.

Eine Seite, die ein Entwurf ist oder gelöscht wurde, steht nicht im
Menü. Ihr Eintrag bleibt im Tab Menu mit einer Warnung, und das Bauen der
Website weist darauf hin, bis Sie eine andere Seite wählen oder den
Eintrag entfernen.

::: faq
[[items]]
question = "Ich habe eine neue Seite angelegt, aber sie steht nicht im Menü."
answer = "Beim automatischen Menü stellen Sie im Tab Settings der Seite **Menu** auf **Main menu**. Bei einem Menü von Hand fügen Sie sie im Tab Menu hinzu: Neue Seiten kommen dort nicht von selbst hin."

[[items]]
question = "Kann eine Gruppe Gruppen enthalten?"
answer = "Nein, nur eine Ebene. So bleibt das Menü auf Handys leicht zu bedienen."
:::
