+++
title = "KI-Assistenten"
description = "Einen KI-Assistenten wie Claude an Ihrer Website arbeiten lassen: was er kann und was nicht, die Einrichtung in Claude Desktop oder Claude Code (Terminal oder VS Code), und wie Sie seine Änderungen prüfen und rückgängig machen."
menu = "main"
weight = 75
summary = "Einen KI-Assistenten wie Claude Seiten für Sie schreiben und ändern lassen, nach den Regeln von q.e.d."
+++

Ein KI-Assistent wie Claude kann für Sie an Ihrer Website arbeiten. Sie
beschreiben in eigenen Worten, was Sie möchten, und er erledigt es in
q.e.d.:

- „Schreib eine Seite über unser Team, mit einem Foto von jedem."
- „Verschieb alle Beiträge aus 2023 unter die Archiv-Seite."
- „Prüf alle Seiten auf Tippfehler und korrigier sie."
- „Welche Bilder werden nirgends verwendet?"

Er arbeitet an derselben Website, die Sie in q.e.d. offen haben, nach
denselben Regeln wie die App. Jede Änderung sehen Sie unter
:icon-history: **Versions** und können sie rückgängig machen.

## Was er kann und was nicht

:::: columns
::: column
title = "Er kann"
boxed = true
:::

- Seiten und Beiträge lesen, schreiben und anlegen, mit allen Blöcken
- einer Seite eine neue Adresse geben (Links und Weiterleitungen
  erledigt q.e.d., wie beim Verschieben in der App)
- Seiten löschen, auf die nichts verlinkt, und Bilder und Dateien, die
  nichts mehr verwendet (etwa um Bildschirmfotos zu ersetzen)
- Bilder und Dateien von Ihrem Computer hinzufügen, auch Dateien, die
  die Website genau so braucht, wie sie sind (etwa alte Foliensätze),
  unter ihren alten Adressen
- die Fußzeile, geteilte Blöcke und die Einstellungen der Website ändern
  (Menü, Logo, Theme, Web-Adresse, Ihre Profile anderswo); die Website
  muss sich damit öffnen lassen, sonst lehnt q.e.d. ab
- die Website bauen und kaputte Links melden
- eine Version mit einer Notiz sichern, was er geändert hat

::: column
title = "Er kann nicht"
boxed = true
:::

- veröffentlichen: die Website online stellen ist immer Ihr Klick, unter
  :icon-upload: **Publish**
- eine fehlerhafte Seite speichern: q.e.d. prüft jede Seite vorher und
  sagt dem Assistenten, was nicht stimmt
- eine Änderung überschreiben, die Sie inzwischen gemacht haben
- etwas außerhalb des Ordners der Website anrühren
::::

## Einrichten

1. Öffnen Sie die Website in q.e.d. und gehen Sie zum Tab
   :icon-settings: **App**. Unter **AI assistants** finden Sie die
   Einstellungen für diese Website, fertig zum Kopieren.
2. Für **Claude Desktop**: Öffnen Sie in Claude Desktop Settings →
   Developer → Edit Config. Das öffnet die Datei
   `claude_desktop_config.json`. Fügen Sie den Teil ein, den q.e.d.
   zeigt (**Copy**), speichern Sie und starten Sie Claude Desktop neu.
   Stehen in der Datei schon Server, ergänzen Sie den Eintrag von
   q.e.d. bei den vorhandenen.
3. Für **Claude Code im Terminal**: Kopieren Sie den Befehl, den q.e.d.
   zeigt, und führen Sie ihn einmal in einem Terminal aus.
4. Für **Claude Code in VS Code**: Der Befehl aus Schritt 3 funktioniert
   dort oft nicht, weil die VS-Code-Erweiterung ihr eigenes Claude Code
   ohne den Befehl `claude` mitbringt. Speichern Sie stattdessen den Teil,
   den q.e.d. unter *Claude Code in VS Code* zeigt, als Datei namens
   `.mcp.json` in dem Ordner, den Sie in VS Code öffnen, zum Beispiel im
   Ordner der Website. Startet Claude Code in diesem Ordner, fragt es
   einmal, ob es den Server darin verwenden soll; bestätigen Sie das.

Das machen Sie einmal für jede Website, an der der Assistent arbeiten
soll. Die Einstellungen nennen q.e.d. selbst: Der Assistent startet es
im Hintergrund, ohne Fenster, und spricht mit ihm. Andere Assistenten,
die das [Model Context Protocol](https://modelcontextprotocol.io)
sprechen, funktionieren genauso.

Wenn Sie q.e.d. oder den Ordner der Website verschieben, richten Sie es
neu ein: Die Einstellungen enthalten beide Orte.

## Seine Arbeit prüfen und rückgängig machen

Die Änderungen des Assistenten werden gespeichert wie Ihre. Bitten Sie
ihn, „am Ende eine Version zu sichern": Seine Notiz steht dann unter
:icon-history: **Versions**, zum Beispiel *„Team-Seite mit Fotos"*.
Dort sehen Sie, was sich auf jeder Seite geändert hat, und können
zurück, wie unter [Versionen](versions.md) beschrieben.

Wenn Sie eine Seite in q.e.d. bearbeiten, während der Assistent
dieselbe Seite ändert, geht nichts verloren. Der Assistent erfährt,
dass sich die Seite geändert hat, und liest sie neu; q.e.d. fragt Sie
beim Speichern, welche Fassung bleiben soll.

Um das Ergebnis vor dem Veröffentlichen zu sehen, nutzen Sie wie immer
die Vorschau-Taste (:icon-eye:) in q.e.d.

::: faq
heading = "Fragen"

[[items]]
question = "Schickt der Assistent meine Website ins Internet?"
answer = "Der Assistent selbst läuft bei seinem Hersteller, bei Claude bei Anthropic. Die Seiten, die er liest und schreibt, gehen als Teil des Gesprächs dorthin, wie alles, was Sie ihm schreiben. q.e.d. selbst schickt nichts irgendwohin."

[[items]]
question = "Muss q.e.d. offen sein?"
answer = "Nein. Der Assistent startet q.e.d. selbst im Hintergrund. Ist q.e.d. auch offen, sehen Sie seine Änderungen, wenn Sie die Seite wieder ansehen."

[[items]]
question = "Kann er aus Versehen veröffentlichen?"
answer = "Nein. Er hat keine Möglichkeit dazu. Sie entscheiden, wann die Website online geht."
:::
