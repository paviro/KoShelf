# ===========================================
#      German (de) — Base Language File
# ===========================================
# This is the base translation file for German, using Standard High German (de_DE).
# Regional variants (e.g., de_AT.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., de_AT.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = de
-lang-name = Deutsch (Deutschland)
-lang-dialect = de_DE

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Bücher
comics = Comics
statistics = Statistiken
calendar = Kalender
recap = Rückblick
settings = Einstellungen
github = GitHub
reading-companion = Lese-Begleiter
# Used in footer/sidebar for update time
last-updated = Letztes Update
view-details = Details anzeigen

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Darstellung
theme-setting = Design
    .description = Wähle, wie KoShelf aussehen soll. Automatisch folgt deiner Systemeinstellung.
    .option-auto = Automatisch
    .option-light = Hell
    .option-dark = Dunkel
prefetch-setting = Links vorladen
    .description = Lädt Seiten vor, sobald du einen Link mit der Maus berührst, ihn fokussierst oder antippst. So geht die Navigation meist schneller.
    .option-enabled = Aktiviert
    .option-disabled = Deaktiviert
    .connection-note = Hinweis: Vorladen wird bei eingeschränkter Verbindung weiterhin automatisch übersprungen (z. B. Datensparmodus oder langsamere Mobilfunknetze).
language-setting = Sprache
    .hint = Legt die Sprache für alle Übersetzungen der Oberfläche fest.
region-setting = Region
    .hint = Beeinflusst die Formatierung von Daten und Zahlen.
    .preview-date = Vorschau Datum
    .preview-number = Vorschau Zahl
    .majority-group = Regionen mit Sprachmehrheit
    .all-group = Alle unterstützten Regionen

# -----------------------------------
#         Authentication
# -----------------------------------
login = Anmelden
    .title = Bei { $site } anmelden
    .password = Passwort
    .submit = Anmelden
    .error = Ungültiges Passwort
    .rate-limited = Zu viele Versuche. Bitte versuche es in Kürze erneut.
change-password = Passwort ändern
    .setting = Passwort
    .current = Aktuelles Passwort
    .new = Neues Passwort
    .confirm = Passwort bestätigen
    .changed = Passwort erfolgreich geändert
    .too-short = Das Passwort muss mindestens { $min } Zeichen lang sein
    .mismatch = Die Passwörter stimmen nicht überein
    .incorrect = Aktuelles Passwort ist falsch
    .current-placeholder = Aktuelles Passwort eingeben
    .new-placeholder = Mind. 8 Zeichen
    .new-hint = Muss mindestens 8 Zeichen lang sein
    .confirm-placeholder = Neues Passwort wiederholen
session-management =
    .setting = Sitzungen
    .current = Aktuelle Sitzung
    .this-device = Dieses Gerät
    .last-active = Zuletzt aktiv
    .revoke = Widerrufen
    .revoke-confirm = Diese Sitzung widerrufen?
    .revoked = Sitzung widerrufen
    .device-info = { $browser } auf { $os }
    .logout = Abmelden

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Suche Buch, Autor, Reihe...
filter =
    .aria-label = Bücher filtern
    .all = Alle
    .all-aria = { filter.aria-label } - Aktuell: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Aktuell: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Aktuell: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Aktuell: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Aktuell: { filter.on-hold }
no-books-found = Keine Bücher gefunden
no-books-match = Keine Bücher entsprechen deiner Suche oder deinem Filter.
try-adjusting = Passe deine Such- oder Filterkriterien an
status =
    .reading = Wird gelesen
    .on-hold = Pausiert
    .completed = Abgeschlossen
    .unread = Ungelesen
book-label = { $count ->
    [one] Buch
   *[other] Bücher
}
comic-label = { $count ->
    [one] Comic
   *[other] Comics
}
books-finished = { $count ->
    [one] { book-label } abgeschlossen
   *[other] { book-label } abgeschlossen
}
comics-finished = { $count ->
    [one] { comic-label } abgeschlossen
   *[other] { comic-label } abgeschlossen
}
unknown-book = Unbekanntes Buch
unknown-author = Unbekannter Autor
by = von
book-overview = Buchübersicht
comic-overview = Comicübersicht

# -----------------------------------
#            Book Details
# -----------------------------------
description = Beschreibung
publisher = Verlag
series = Reihe
genres = Genres
language = Sprache
book-identifiers = Buch-Identifikatoren
my-review = Meine Rezension
my-note = Meine Notiz
highlights = Markierungen
highlights-label = { $count ->
    [one] Markierung
   *[other] Markierungen
}
notes-label = { $count ->
    [one] Notiz
   *[other] Notizen
}
bookmarks = Lesezeichen
page-bookmark = Seiten-Lesezeichen
bookmark-anchor = Lesezeichen-Anker
highlights-quotes = Markierungen & Zitate
additional-information = Zusätzliche Informationen
reading-progress = Lesefortschritt
page-number = Seite { $count }
last-read = Zuletzt gelesen
pages = { $count ->
    [one] { $count } Seite
   *[other] { $count } Seiten
}
pages-label = { $count ->
    [one] Seite
   *[other] Seiten
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Lesestatistiken
overall-statistics = Gesamtstatistiken
weekly-statistics = Wochenstatistiken
yearly-statistics = Jährliche Statistiken
total-read-time = Gesamte Lesezeit
total-pages-read = Gesamte gelesene Seiten
pages-per-hour = Seiten/Stunde
# Abbreviation for Pages Per Hour
pph-abbreviation = S/h
reading-sessions-label = { $count ->
    [one] Lese-Sitzung
   *[other] Lese-Sitzungen
}
session =
    .longest = Längste Sitzung
    .average = Ø Sitzung
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /Ø Sitzung
streak =
    .current = Aktueller Streak
    .longest = Längster Streak
reading-streak = Lesestreak
average-time-day = Ø Zeit/Tag
average-pages-day = Ø Seiten/Tag
most-pages-in-day = Meiste Seiten an einem Tag
longest-daily-reading = Längste tägliche Lesezeit
reading-completions = Abgeschlossene Lesungen
completed-books = Abgeschlossene Bücher
statistics-from-koreader = Statistiken aus KoReader Lese-Sitzungen
reading-time = Lesezeit
pages-read = Gelesene Seiten
units-days = { $count ->
    [one] { $count } Tag
   *[other] { $count } Tage
}
units-sessions = { $count ->
    [one] { $count } Sitzung
   *[other] { $count } Sitzungen
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Mein KoShelf Lese-Rückblick
share = Teilen
    .recap-label = Rückblickbild teilen
download = Herunterladen
    .recap-label = Rückblickbild herunterladen
recap-story = Story
    .details = 1260 x 2240 — Vertikal 9:16
recap-square = Quadrat
    .details = 1500 x 1500 — Quadrat 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Bester Monat
active-days = { $count ->
    [one] Aktiver Tag
   *[other] Aktive Tage
}
active-days-tooltip = { $count ->
    [one] aktiver Tag
   *[other] aktive Tage
}
toggle =
    .hide = Ausblenden
    .show = Anzeigen
less = Weniger
more = Mehr
period = Zeitraum
sessions = Sitzungen
yearly-summary = Jahreszusammenfassung { $count }
recap-empty =
    .nothing-here = Hier gibt es noch nichts
    .try-switching = Versuche, den Bereich oder das Jahr oben zu wechseln.
    .finish-reading = Beende ein Buch in KoReader, um deinen Rückblick zu sehen.
    .info-question = Warum wird mein Rückblick nicht angezeigt?
    .info-answer = KoShelf verwendet Lesestatistiken zur Erkennung von abgeschlossenen Büchern und Comics, was das Nachverfolgen von Wiederholungslektüren ermöglicht. Ein Buch einfach als „beendet" zu markieren, ohne Lesedaten zu haben, lässt es hier nicht erscheinen.
stats-empty =
    .nothing-here = Hier gibt es noch nichts
    .start-reading = Beginne mit KoReader zu lesen, um deine Statistiken hier zu sehen.
    .info-question = Wie funktioniert die Leseerfassung?
    .info-answer = KoReader erfasst automatisch deine Lesesitzungen, einschließlich der verbrachten Zeit und gelesenen Seiten. Synchronisiere deine Statistik-Datenbank mit KoShelf, um deine Aktivitäten hier visualisiert zu sehen.
error-state =
    .title = Etwas ist schiefgelaufen
    .description =
        Die Daten konnten nicht geladen werden.
        Bitte versuche es erneut.
    .not-found-title = Nicht gefunden
    .not-found-description = Die Seite, die du suchst, existiert nicht oder wurde entfernt.
    .connection-title = Verbindung fehlgeschlagen
    .connection-description =
        Der Server konnte nicht erreicht werden.
        Überprüfe deine Verbindung und versuche es erneut.
    .file-unavailable-title = Buchdatei nicht verfügbar
    .file-unavailable-description = Die Buchdetails wurden gefunden, aber die Buchdatei fehlt.
    .retry = Erneut versuchen

# Navigation and sorting
sort-order =
    .aria-label-toggle = Sortierreihenfolge umschalten
    .newest-first = { sort-order.aria-label-toggle } - Aktuell: Neueste zuerst
    .oldest-first = { sort-order.aria-label-toggle } - Aktuell: Älteste zuerst
previous-month =
    .aria-label = Vorheriger Monat
next-month =
    .aria-label = Nächster Monat
search =
    .aria-label = Suchen
close-search =
    .aria-label = Suche schließen
go-back =
    .aria-label = Zurück

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Leseansicht
    .loading = Buch wird geladen…
    .previous-page = Vorherige Seite
    .next-page = Nächste Seite
    .contents = Inhalt
open-at-annotation = An Markierung öffnen
open-in-reader = Öffnen
    .aria-label = In der Leseansicht öffnen
reader-settings =
    .aria-label = Anzeigeeinstellungen
    .typography = Typografie
    .margins = Seitenränder
reader-font-size = Schriftgröße
    .decrease-aria = Schriftgröße verringern
    .increase-aria = Schriftgröße erhöhen
reader-line-spacing = Zeilenabstand
    .decrease-aria = Zeilenabstand verringern
    .increase-aria = Zeilenabstand erhöhen
reader-word-spacing = Wortabstand
    .decrease-aria = Wortabstand verringern
    .increase-aria = Wortabstand erhöhen
reader-hyphenation = Silbentrennung
reader-floating-punctuation = Hängende Interpunktion
reader-embedded-fonts = Eingebettete Schriften
reader-left-margin = Linker Rand
    .decrease-aria = Linken Rand verkleinern
    .increase-aria = Linken Rand vergrößern
reader-right-margin = Rechter Rand
    .decrease-aria = Rechten Rand verkleinern
    .increase-aria = Rechten Rand vergrößern
reader-top-margin = Oberer Rand
    .decrease-aria = Oberen Rand verkleinern
    .increase-aria = Oberen Rand vergrößern
reader-bottom-margin = Unterer Rand
    .decrease-aria = Unteren Rand verkleinern
    .increase-aria = Unteren Rand vergrößern
reader-mode =
    .auto = Buch
    .on = Ein
    .off = Aus
reader-reset =
    .book = Bucheinstellungen verwenden
    .book-aria = Auf die synchronisierten Anzeigeeinstellungen des Buches zurücksetzen
    .defaults = Auf Standard zurücksetzen
    .defaults-aria = Anzeigeeinstellungen des Readers auf Standard zurücksetzen
reader-drawer =
    .aria-label = Buch-Navigationsbereich
reader-no-toc = Kein Inhaltsverzeichnis verfügbar
    .description = Diese Datei enthält keine Kapitelmarken.
reader-no-highlights = Noch keine Markierungen
    .description = Markierungen, die du in KoReader anlegst, werden hier angezeigt.
reader-no-bookmarks = Noch keine Lesezeichen
    .description = Lesezeichen, die du in KoReader anlegst, werden hier angezeigt.
select-month = Monat auswählen

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Januar
    .short = Jan
february = Februar
    .short = Feb
march = März
    .short = Mär
april = April
    .short = Apr
may = Mai
    .short = Mai
june = Juni
    .short = Jun
july = Juli
    .short = Jul
august = August
    .short = Aug
september = September
    .short = Sep
october = Oktober
    .short = Okt
november = November
    .short = Nov
december = Dezember
    .short = Dez

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Mo
    .tue = Di
    .wed = Mi
    .thu = Do
    .fri = Fr
    .sat = Sa
    .sun = So

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = W
    .d = T
    .h = h
    .m = min
today = Heute
of-the-year = des Jahres
of = von
last = Zuletzt

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] Tag
   *[other] Tage
}
hours_label = { $count ->
    [one] Stunde
   *[other] Stunden
}
minutes_label = { $count ->
    [one] Minute
   *[other] Minuten
}
