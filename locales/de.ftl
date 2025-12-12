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
statistics = Statistiken
calendar = Kalender
recap = Rückblick
github = GitHub
loading = Laden...
reload = Neu laden
new-version-available = Neue Version verfügbar
tap-to-reload = Tippen zum Neuladen
reading-companion = Lese-Begleiter
# Used in footer/sidebar for update time
last-updated = Letztes Update
view-details = Details anzeigen

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
books-finished = { $count ->
    [one] { book-label } abgeschlossen
   *[other] { book-label } abgeschlossen
}
unknown-book = Unbekanntes Buch
unknown-author = Unbekannter Autor
by = von
book-overview = Buchübersicht

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
notes = Notizen
notes-label = { $count ->
    [one] Notiz
   *[other] Notizen
}
bookmarks = Lesezeichen
page = Seite
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
days-read = Gelesene Tage
weekly-reading-time = Wöchentliche Lesezeit
weekly-pages-read = Wöchentliche Seiten
average-time-day = Ø Zeit/Tag
average-pages-day = Ø Seiten/Tag
most-pages-in-day = Meiste Seiten an einem Tag
longest-daily-reading = Längste tägliche Lesezeit
reading-completions = Abgeschlossene Lesungen
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
    .details = 2160 x 2160 — Quadrat 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Bester Monat
active-days = { $count ->
    [one] Aktiver Tag
   *[other] Aktive Tage
}
toggle =
    .hide = Ausblenden
    .show = Anzeigen
less = Weniger
more = Mehr
period = Zeitraum
sessions = Sitzungen
yearly-summary = Jahreszusammenfassung { $count }

# Navigation and sorting
previous-year =
    .aria-label = Vorheriges Jahr
next-year =
    .aria-label = Nächstes Jahr
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
close = Schließen
    .aria-label = Schließen
go-back =
    .aria-label = Zurück

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
    .thu = Do
    .sun = So

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime =
    .full = %-d. %B %Y um %H:%M Uhr
    .short-current-year = %-d. %b
    .short-with-year = %-d. %b %Y

# Time units: h=hours, m=minutes, d=days
units =
    .h = h
    .m = min
    .d = d
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