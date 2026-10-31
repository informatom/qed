+++
title = "Veröffentlichen"
description = "Die Website über SFTP auf Ihren eigenen Webspace bringen: einmal ein Ziel einrichten, die Verbindung testen, dann mit einer Notiz zu den Neuigkeiten veröffentlichen."
menu = "main"
weight = 60
summary = "Die Website auf Ihren Webspace bringen: einmal ein Ziel einrichten, dann mit einem Klick veröffentlichen."
+++

q.e.d. baut die Website auf Ihrem Computer und kopiert sie auf Ihren
Webspace, den Platz für eine Website, den Ihnen Ihr Hosting-Anbieter
gibt. Es verwendet SFTP, eine sichere Art, Dateien zu kopieren, die fast
jeder Hosting-Anbieter bietet. Auf dem Server läuft nichts: Die Website
besteht aus einfachen Dateien, schnell und günstig zu hosten.

## Vor dem ersten Mal

Ihr Hosting-Anbieter gibt Ihnen vier Dinge, meist in seinem Kundenbereich
oder in der E-Mail von der Anmeldung:

- den **Server**, etwa `ssh.example.com`
- Ihren **Benutzernamen**
- ein **Passwort**, oder die Möglichkeit, sich mit einem Schlüssel
  anzumelden
- den **Ordner auf dem Server**, in den die Website kommt, etwa `/www`
  oder `/home/sie/public_html`

Tragen Sie außerdem die **Web address** der Website im Tab **Site** ein:
Feeds, Link-Vorschauen und die Sitemap brauchen sie.

## Ein Ziel einrichten

Ein *Ziel* ist ein Ort, an den die Website veröffentlicht wird. Im Tab
**Publish** :icon-plus: **Add target** und die vier Dinge ausfüllen, mit
einem **Name** dafür, etwa *Website*. Mit **Log in with: Key** macht
q.e.d. einen Schlüssel für diesen Server; kopieren Sie ihn in die
Einstellungen Ihres Hosting-Anbieters für SSH-Schlüssel (oder in
`~/.ssh/authorized_keys` auf dem Server). Dann **Save target**.

Passwörter und Schlüssel liegen im Schlüsselbund Ihres Computers, nie im
Website-Ordner, damit sie nicht in Sicherungen oder auf anderen
Computern landen.

## Testen, dann veröffentlichen

::: image
image = "manual/publish-tab.png"
alt = "Der Tab Publish: eine Schaltfläche Build, und unter Publishing ein Ziel namens Alpha mit den Abzeichen key und server trusted, und den Schaltflächen Edit, Remove, Test connection und Publish. Die Adresse des Ziels ist unscharf gemacht."
caption = "Ein getestetes Ziel: **server trusted**, bereit für **Publish**."
:::

1. **Test connection.** Beim ersten Mal zeigt q.e.d. den Fingerabdruck
   des Server-Schlüssels, seinen Ausweis. Veröffentlicht Ihr
   Hosting-Anbieter ihn, vergleichen Sie; dann **Trust**. Ab jetzt
   bemerkt q.e.d., wenn unter dieser Adresse ein anderer Server
   antwortet.
2. :icon-upload: **Publish.** q.e.d. fragt *What's new on the site?*:
   ein paar Worte für die Geschichte, etwa *Sommeröffnungszeiten*. Dann
   baut es die Website und kopiert nur die Dateien der Website, die sich
   von denen auf dem Server unterscheiden.
3. **Open the website**, um sie online anzusehen.

Dateien, die q.e.d. nicht selbst hochgeladen hat, rührt es auf dem
Server nie an; andere Dinge im selben Ordner bleiben also, wie sie sind.

::: faq
[[items]]
question = "Publish sagte, 3 meiner Dateien hätten sich geändert, hat aber 57 hochgeladen. Warum?"
answer = "Ihre Dateien und die Dateien der Website sind zweierlei. Der Dialog Publish listet die Dateien, die Sie in q.e.d. geändert haben, etwa eine Seite. Die Website wird aus allen gebaut, und eine Änderung kann sich auf vielen ihrer Seiten zeigen: Der Titel einer Seite steht im Menü jeder Seite, ein neuer Beitrag in den Post lists und in den Links zurück und weiter. Publish lädt jede Datei der Website hoch, die sich von der auf dem Server unterscheidet, daher ist die Zahl nach dem Veröffentlichen (*57 website files uploaded*) oft größer."

[[items]]
question = "Muss ich zuerst bauen?"
answer = "Nein. Publish baut die Website selbst. :icon-hammer: **Build** im Tab Publish ist dafür da, die Dateien anzusehen oder selbst hochzuladen."

[[items]]
question = "Kann ich die Website sehen, bevor sie online geht?"
answer = "Ja: Die Schaltfläche :icon-eye: oben rechts zeigt sie in Ihrem Browser, nur von Ihrem Computer aus."

[[items]]
question = "Publish wartet auf eine Episode. Warum?"
answer = "Auf einer Podcast-Website braucht jede veröffentlichte Episode ihre Audiodatei, in jedem Format, das der Podcast anbietet, sonst bekämen Podcast-Apps eine kaputte Episode. Fügen Sie das Audio hinzu, oder markieren Sie die Episode als Entwurf, bis sie fertig ist."

[[items]]
question = "Ist die alte Website weg, wenn ich veröffentliche?"
answer = "q.e.d. ersetzt, was es vorher dort abgelegt hat, und fügt Neues hinzu. Dateien, die es nicht dort abgelegt hat, bleiben. Jedes Veröffentlichen wird außerdem in der Geschichte der Website vermerkt, Sie sehen also, was wann online war."
:::
