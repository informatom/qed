+++
title = "Eine Website umziehen"
description = "Eine Website oder einen Podcast von einem anderen Computer oder System zu q.e.d. bringen, die alten Adressen funktionierend halten und umstellen, wenn alles bereit ist."
menu = "main"
weight = 80
summary = "Von einem anderen Computer oder System, ohne Besucher zu verlieren."
+++

## Von einem anderen Computer

Eine q.e.d.-Website zieht mit ihrem Ordner um. Ist sie auf einem
Git-Server gesichert, holt **Get a site from git** sie mit allen
Versionen herüber (siehe [Sicherungen](backups.md)); eine Kopie des
Ordners, etwa von einem USB-Stick, öffnen Sie mit **Open site folder…**.

## Von einem anderen System

Einen Import aus WordPress, Jekyll oder Baukasten-Websites gibt es
nicht: Eine Website zieht von Hand um, Seite für Seite. Das ist auch
ein guter Moment zum Aufräumen.

1. **Legen Sie eine neue Website an** (**New site…**), mit demselben
   Titel und derselben Sprache, und füllen Sie die
   [Einstellungen der Seite](site-settings.md) aus, mit der Webadresse,
   die die Website haben wird.
2. **Bringen Sie die Bilder und Dateien mit**: Bilder in Media, Dateien
   zum Herunterladen in Downloads
   ([Bilder und Dateien](pictures-and-files.md)).
3. **Legen Sie die Seiten an**, jede möglichst unter ihrer alten
   Adresse: Der Dialog **New page** zeigt die Adresse, die sie bekommt,
   und :icon-folder-down: verschiebt oder benennt eine Seite um.
   Kopieren Sie den Text herüber und fügen Sie Überschriften, Links und
   Bilder wieder ein.
4. **Bauen Sie das Aussehen nach**: ein Theme, die Farben Ihres Logos
   und Ihre Schriften ([Farben und Schriften](colors-and-fonts.md)),
   und das Menü ([Menüs](menus.md)).
5. **Vorschau** (:icon-eye:) und durchklicken. Die Liste fehlender Links
   zeigt, was noch zu tun ist.

Dateien, die genau ihre alte Adresse behalten müssen, etwa
`/pdf/preisliste.pdf`, kommen nach **Static**, in einen Ordner
desselben Namens: Die Website liefert sie aus, wie sie sind.

## Alte Adressen

Links von anderen Websites, Suchmaschinen und Lesezeichen zeigen auf die
alten Adressen. Wo eine Seite ihre Adresse behält, geht nichts kaputt.
Wird eine Seite in q.e.d. verschoben oder umbenannt, führt die alte
Adresse von selbst zur neuen.

Alte Adressen, die eine q.e.d.-Seite nicht haben kann, etwa solche, die
auf `.html` enden, lassen sich auch erhalten: Öffnen Sie die Seite im Tab
**Pages**, dann ihren Tab **Settings**; drücken Sie unter **Old
addresses** **Add an address** und tippen Sie sie ein
(`/2022/11/11/episode51.html`), oder fügen Sie die vollständige Adresse
der alten Website ein; q.e.d. behält nur den Teil nach der Domain.
Besucher, die dort ankommen, werden zur Seite weitergeschickt. Dieselbe
Liste zeigt die Adressen, die q.e.d. beim Verschieben der Seite behalten
hat; **Remove** nimmt eine weg.

Eine Adresse kann nur zu einer Seite führen: q.e.d. speichert keine, die
schon die Adresse einer anderen Seite ist, oder eine alte Adresse einer
anderen Seite.

## Ein Podcast

Podcast-Apps müssen neue Episoden weiter finden und sollen die alten
nicht als neu zeigen.

- **Von jekyll-octopod**: Die Feeds von q.e.d. liegen unter denselben
  Adressen (`/episodes.mp3.rss`) und sein Audio auch
  (`/episodes/episode1.mp3`), daher bemerken Abonnenten nichts. 3
  Schweinehunde ist so umgezogen.
- **Von anderen Systemen** (Podlove Publisher, ein Podcast-Hoster): Die
  Feeds hatten andere Adressen, und die Identitäten der Episoden
  (*GUIDs*) müssen übernommen werden, damit Apps nicht jede Episode als
  neu zeigen. Drücken Sie im Tab **Settings** jeder Episode (im Tab
  **Pages**, neben **Content**) **Change it** neben ihrer GUID und
  tippen Sie die aus dem alten Feed ein (die `<guid>` im `<item>` der
  Episode), genau so, wie sie dort steht. Ein Import, der das von selbst
  erledigt, ist geplant; die alten Feed-Adressen brauchen vorerst noch
  Hilfe.

## Umstellen

Veröffentlichen Sie die neue Website auf Ihrem Webspace, sobald alles an
seinem Platz ist ([Veröffentlichen](publishing.md)). Geht sie auf einen
neuen Webspace, lässt der Anbieter Ihrer Domain die Domain dorthin
zeigen; bis dahin bleibt die alte Website online.
