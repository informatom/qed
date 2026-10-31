+++
title = "Sicherungen"
description = "Wie Sie eine q.e.d.-Website sichern und auf zwei Computern bearbeiten: Push und Pull ihres Git-Repositorys, und was dabei nicht dabei ist."
menu = "main"
weight = 50
+++

Eine q.e.d.-Website ist ein Ordner, und jeder Website-Ordner ist ein
[Git](https://git-scm.com)-Repository: q.e.d. hält dort jede Version
fest (siehe Tab *Versions*). Eine Sicherung ist also eine Kopie dieses
Repositorys, und die beste Kopie liegt auf einem anderen Computer,
gemacht mit `git push`. Sie enthält nicht nur den heutigen Stand,
sondern jede Version, und Sie können jede davon zurückholen.

## Eine Sicherung einrichten

Legen Sie einmal pro Website anderswo ein leeres Repository an: auf
[Codeberg](https://codeberg.org) oder [GitHub](https://github.com) (ein
privates Repository genügt), oder auf Ihrem eigenen Server
(`git init --bare /var/git/meine-website.git`). Dann, im Tab *Versions*
unter *Push and pull*:

1. Fügen Sie die SSH-Adresse des Repositorys hinzu (auf Codeberg oder
   GitHub: die Adresse „SSH“ unter *Clone*), zum Beispiel
   `git@codeberg.org:ich/meine-website.git`.
2. *Make a key*, kopieren Sie ihn und fügen Sie ihn auf dem Server
   hinzu: in den Einstellungen des Repositorys als *Deploy Key* mit
   Schreibrecht, oder auf Ihrem eigenen Server in
   `~/.ssh/authorized_keys`. Der private Schlüssel bleibt im
   Schlüsselbund Ihres Computers.
3. *Push*. Beim ersten Mal zeigt q.e.d. den Fingerabdruck des
   Server-Schlüssels; vertrauen Sie ihm, wenn er mit dem übereinstimmt,
   was der Betreiber des Servers veröffentlicht.

## Eine Sicherung machen

*Push* im Tab Versions. Es schließt zuerst die offene Version (mit der
Notiz im Feld *Save a version*, falls Sie eine getippt haben), damit
die letzten Änderungen in der Sicherung sind. Auch das Veröffentlichen
schließt die offene Version, daher ist ein Push nach jedem
Veröffentlichen eine gute Gewohnheit.

## Zwei Computer

Wählen Sie auf dem zweiten Computer im Website-Menü (oder auf dem
Startbildschirm) *Get a site from git…*, geben Sie dieselbe Adresse ein
und machen Sie auch dort einen Schlüssel (ein Schlüssel pro Computer).
q.e.d. kopiert die Website mit allen Versionen. Ab dann: Push, wenn Sie
auf einem Computer fertig sind, Pull, bevor Sie auf dem anderen
beginnen. Wenn sich eine Website öffnet, fragt q.e.d. beim entfernten
Repository nach, ob es neue Versionen hat, und bietet an, sie zu holen.

Haben Sie auf beiden Computern gearbeitet, ohne dazwischen zu holen,
führt *Pull* die Versionen zusammen, solange verschiedene Dateien
geändert wurden. Hat sich dieselbe Seite auf beiden geändert, ändert
sich nichts, und q.e.d. nennt die Dateien: Behalten Sie eine Seite, oder
lösen Sie es mit Git.

## Wiederherstellen

Auf jedem Computer *Get a site from git…*, oder auf der Kommandozeile:

```sh
git clone git@codeberg.org:ich/meine-website.git meine-website
```

Öffnen Sie den Ordner in q.e.d., und alles ist da: Seiten, Medien,
Dateien zum Herunterladen, statische Dateien, Schriften, Einstellungen
und die ganze Versionsgeschichte.

## Mit Git auf der Kommandozeile

Die entfernten Repositorys sind die von Git selbst, daher funktionieren
auch `git remote add backup …` und `git push backup main` und erscheinen
in der App. q.e.d. macht Push und Pull nur für Websites, die ein
eigenes Git-Repository sind; für einen Website-Ordner in einem größeren
Repository verwenden Sie Git dort.

## Was nicht in der Sicherung ist

- **Ziele zum Veröffentlichen** (`targets.toml`) gehören absichtlich
  nicht zur Geschichte. Richten Sie die Ziele nach dem Wiederherstellen
  unter *Publishing* neu ein. Sie enthalten nur Servernamen und Ordner,
  nichts Geheimes.
- **Passwörter und SSH-Schlüssel** liegen im Schlüsselbund Ihres
  Computers, nie im Website-Ordner. Sichern Sie sie so, wie Sie Ihre
  anderen Passwörter sichern, oder machen Sie auf einem neuen Computer
  einen neuen Schlüssel und fügen Sie ihn auf dem Server hinzu.
- **Die gebaute Website** (`_site/`) und der Bild-Cache von q.e.d.
  (`.qed-cache/`) bleiben absichtlich draußen: Das nächste Bauen macht
  sie neu.

## Ohne Git

Den ganzen Website-Ordner (samt dem versteckten Ordner `.git`) auf ein
Sicherungslaufwerk zu kopieren, funktioniert auch und bewahrt die
Geschichte ebenso. Es wird nur leichter vergessen als ein Push nach dem
Veröffentlichen.
