+++
title = "Bilder und Dateien"
description = "Fotos und Logos in Media, Dateien zum Herunterladen in Downloads, und Dateien, die die Website so braucht, wie sie sind, in Static ablegen und auf Seiten verwenden."
menu = "main"
weight = 30
summary = "Fotos und Logos, Dateien zum Herunterladen, und Dateien, die die Website so braucht, wie sie sind."
+++

Eine Website hat ihre Dateien an drei Orten, jeder mit eigenem Tab:

:::: columns
::: column
title = "Media"
boxed = true
:::

:icon-photo: Bilder: Fotos, Logos, Illustrationen. JPEG, PNG, GIF, WebP
und SVG, und ICO für das [Website-Icon](site-settings.md).

::: column
title = "Downloads"
boxed = true
:::

:icon-download: Dateien, die Besucher herunterladen: PDFs, Dokumente,
Videodateien. Jeder Typ.

::: column
title = "Static"
boxed = true
:::

:icon-folder: Dateien, die die Website genau so braucht, wie sie sind,
unter ihrer eigenen Adresse: eine Präsentation, eine
Verifizierungsdatei.
::::

## Dateien hinzufügen

**Import images…** (oder **Import files…**) kopiert Dateien von Ihrem
Computer in die Website. q.e.d. gibt ihnen saubere Namen: klein
geschrieben, ohne Leerzeichen, damit sie als Webadressen funktionieren.
Die Originale bleiben, wo sie waren. **Show in file manager** öffnet den
Ordner selbst.

Jede Datei zeigt ihre Größe und welche Seiten sie verwenden; Dateien,
die niemand verwendet, sind mit **Not used** markiert. Nur diese lassen
sich löschen (:icon-trash:), damit keine Seite plötzlich ohne Bild
dasteht.

## Bilder auf Seiten

Die Blöcke **Image**, **Text with image**, **Gallery** und **Hero**
wählen ein Bild aus Media mit einem Klick. Im Text schreiben Sie ein
Bild als `![was es zeigt](/media/foto.jpg)`.

Füllen Sie immer den **Alternativtext** aus: was das Bild zeigt, in
einem Satz. Menschen, die es nicht sehen können, hören ihn vorgelesen,
und Suchmaschinen lesen ihn auch.

::: text-image
text = """
**Große Fotos sind kein Problem.** q.e.d. macht beim Bauen der Website
kleinere Kopien jedes Fotos, und der Browser jedes Besuchers lädt die,
die zu seinem Bildschirm passt. Ein Handy lädt nicht die 4000 Pixel
eines Kamerafotos herunter, um 400 zu zeigen.

Große Bilder öffnen sich auf einen Klick groß über der Seite, ohne sie
zu verlassen."""
image = "manual/media-tab.png"
alt = "Der Tab Media von q.e.d.: ein Suchfeld, die Schaltfläche Import images, und die Bilder der Website als Kacheln, jedes mit Name, Größe, Pixelmaßen und den Seiten, die es verwenden."
image_side = "right"
:::

## Wo Bilder beschnitten werden

Galerien, Kacheln in Beitrags- und Seitenlisten und der Block Hero
zeigen Bilder zugeschnitten, eine Gallery als Quadrate. q.e.d. schneidet
um die Mitte herum, und das passt für die meisten Bilder. Wenn nicht,
etwa bei einem Porträt, dem der Kopf fehlt, wählen Sie die Stelle
selbst: Klicken Sie das Bild im Tab Media an, damit es groß erscheint,
klicken Sie auf :icon-crop: **Crop** und dann auf die Stelle, die
sichtbar bleiben muss. Ein Quadrat daneben zeigt das Ergebnis, und die
Wahl gilt überall, wo das Bild verwendet wird. :icon-focus-centered:
**Middle** schneidet wieder um die Mitte.

## Dateien zum Herunterladen

Der Block **Download** bietet eine Datei aus Downloads an, mit Typ und
Größe. Im Text verlinken Sie sie als `[die Speisekarte](/assets/speisekarte.pdf)`.
Auch ein Block **Video** kann eine Videodatei aus Downloads abspielen.

## Die ganze Website als PDF

:icon-file-type-pdf: **Make PDF of the site** im Tab Downloads macht aus
der ganzen Website ein PDF, zum Ausdrucken, zum Archivieren oder zum
Herunterladen. Es beginnt mit einem Titelblatt und einem
Inhaltsverzeichnis der obersten zwei Ebenen von Seiten, mit
Seitenzahlen; PDF-Programme zeigen sie auch als Lesezeichen. Dann folgen
die Seiten in der Reihenfolge der Website, jede Seite der obersten Ebene
auf einer neuen Seite, und nach ihnen die Beiträge und Episoden, die
neuesten zuerst, in einem eigenen Kapitel nach der Seite, die sie
auflistet (etwa dem Blog).

Das PDF ist schlichter schwarzer Text auf Weiß, ohne die Farben,
Schriften und das Logo der Website. Bilder, Tabellen, Formeln, Diagramme
und Spalten kommen mit; Links zu anderen Seiten springen innerhalb des
PDFs, und Links zu anderen Websites zeigen ihre Adresse in einer
Fußnote, damit man sie auch auf Papier lesen kann. Der Player und Videos
werden zu einem Link auf ihre Seite der Website.

Es entsteht nur, wenn Sie klicken, und landet wie jede andere Datei in
Downloads: `meine-seite.pdf`, nach dem Titel der Website benannt. Bieten
Sie es mit einem Block **Download** an. Eine Website in mehreren
Sprachen bekommt ein PDF pro Sprache (`meine-seite-en.pdf`). Klicken Sie
nach Änderungen an den Seiten noch einmal, um es auf den neuesten Stand
zu bringen; Entwürfe und Beiträge mit einem Datum in der Zukunft bleiben
draußen.

::: faq
[[items]]
question = "Meine PDFs liegen in Media. Ist das ein Problem?"
answer = "Nein, sie funktionieren weiter. Media ist aber für Bilder gedacht; **Move to Downloads** im Tab Media verschiebt sie und passt die Seiten an, die auf sie verlinken."

[[items]]
question = "Darf ich ein Bild von einer anderen Website verwenden?"
answer = "Nur wenn Sie es dürfen, und dann besser als Kopie in Media als über einen Link: q.e.d.-Websites laden nichts von anderen Websites, so bleiben sie schnell und geben die Daten Ihrer Besucher an niemanden weiter."
:::
