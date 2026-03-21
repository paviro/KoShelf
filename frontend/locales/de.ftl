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
appearance-setting = Darstellung
theme-setting = Design
theme-setting-description = Wähle, wie KoShelf aussehen soll. Automatisch folgt deiner Systemeinstellung.
theme-option-auto = Automatisch
theme-option-light = Hell
theme-option-dark = Dunkel
prefetch-setting = Links vorladen
prefetch-setting-description = Lädt Seiten vor, sobald du einen Link mit der Maus berührst, ihn fokussierst oder antippst. So geht die Navigation meist schneller.
prefetch-option-enabled = Aktiviert
prefetch-option-disabled = Deaktiviert
prefetch-setting-connection-note = Hinweis: Vorladen wird bei eingeschränkter Verbindung weiterhin automatisch übersprungen (z. B. Datensparmodus oder langsamere Mobilfunknetze).
language-setting = Sprache
region-setting = Region
language-setting-hint = Legt die Sprache für alle Übersetzungen der Oberfläche fest.
region-setting-hint = Beeinflusst die Formatierung von Daten und Zahlen.
preview-date = Vorschau Datum
preview-number = Vorschau Zahl
region-setting-majority-group = Regionen mit Sprachmehrheit
region-setting-all-group = Alle unterstützten Regionen
login = Anmelden
login-title = Bei { $site } anmelden
login-password = Passwort
login-submit = Anmelden
login-error = Ungültiges Passwort
login-rate-limited = Zu viele Versuche. Bitte versuche es in Kürze erneut.
password-setting = Passwort
change-password = Passwort ändern
current-password = Aktuelles Passwort
new-password = Neues Passwort
confirm-password = Passwort bestätigen
password-changed = Passwort erfolgreich geändert
password-too-short = Das Passwort muss mindestens { $min } Zeichen lang sein
password-mismatch = Die Passwörter stimmen nicht überein
incorrect-password = Aktuelles Passwort ist falsch
sessions-setting = Sitzungen
current-session = Aktuelle Sitzung
this-device = Dieses Gerät
last-active = Zuletzt aktiv
revoke-session = Widerrufen
revoke-session-confirm = Diese Sitzung widerrufen?
session-revoked = Sitzung widerrufen
session-device-info = { $browser } auf { $os }
logout = Abmelden
current-password-placeholder = Aktuelles Passwort eingeben
new-password-placeholder = Mind. 8 Zeichen
new-password-hint = Muss mindestens 8 Zeichen lang sein
confirm-password-placeholder = Neues Passwort wiederholen

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
open-reader-aria = In der Leseansicht öffnen
reader-title = Leseansicht
reader-loading = Buch wird geladen…
reader-previous-page = Vorherige Seite
reader-next-page = Nächste Seite
open-at-annotation = An Markierung öffnen
reader-contents = Inhalt
reader-settings-aria = Anzeigeeinstellungen
reader-section-text = Text
reader-section-typography = Typografie
reader-section-margins = Seitenränder
reader-font-size = Schriftgröße
reader-font-size-decrease-aria = Schriftgröße verringern
reader-font-size-increase-aria = Schriftgröße erhöhen
reader-line-spacing = Zeilenabstand
reader-line-spacing-decrease-aria = Zeilenabstand verringern
reader-line-spacing-increase-aria = Zeilenabstand erhöhen
reader-word-spacing = Wortabstand
reader-word-spacing-decrease-aria = Wortabstand verringern
reader-word-spacing-increase-aria = Wortabstand erhöhen
reader-hyphenation = Silbentrennung
reader-floating-punctuation = Hängende Interpunktion
reader-embedded-fonts = Eingebettete Schriften
reader-left-margin = Linker Rand
reader-left-margin-decrease-aria = Linken Rand verkleinern
reader-left-margin-increase-aria = Linken Rand vergrößern
reader-right-margin = Rechter Rand
reader-right-margin-decrease-aria = Rechten Rand verkleinern
reader-right-margin-increase-aria = Rechten Rand vergrößern
reader-top-margin = Oberer Rand
reader-top-margin-decrease-aria = Oberen Rand verkleinern
reader-top-margin-increase-aria = Oberen Rand vergrößern
reader-bottom-margin = Unterer Rand
reader-bottom-margin-decrease-aria = Unteren Rand verkleinern
reader-bottom-margin-increase-aria = Unteren Rand vergrößern
reader-mode-auto = Buch
reader-mode-on = Ein
reader-mode-off = Aus
reader-reset-book = Bucheinstellungen verwenden
reader-reset-book-aria = Auf die synchronisierten Anzeigeeinstellungen des Buches zurücksetzen
reader-reset-defaults = Auf Standard zurücksetzen
reader-reset-defaults-aria = Anzeigeeinstellungen des Readers auf Standard zurücksetzen
reader-drawer-aria = Buch-Navigationsbereich
reader-no-toc = Kein Inhaltsverzeichnis verfügbar
reader-no-toc-description = Diese Datei enthält keine Kapitelmarken.
reader-no-highlights = Noch keine Markierungen
reader-no-highlights-description = Markierungen, die du in KoReader anlegst, werden hier angezeigt.
reader-no-bookmarks = Noch keine Lesezeichen
reader-no-bookmarks-description = Lesezeichen, die du in KoReader anlegst, werden hier angezeigt.
open-in-reader = Öffnen
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
