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
filter-all = Alle
filter-reading = Lesend
filter-completed = Abgeschlossen
filter-unread = Ungelesen
no-books-found = Keine Bücher gefunden
no-books-match = Keine Bücher entsprechen deiner Suche oder deinem Filter.
try-adjusting = Passe deine Such- oder Filterkriterien an
currently-reading = Wird gelesen
on-hold = Pausiert
# Also used as status
completed = Abgeschlossen
# Also used as status
unread = Ungelesen
book-label = { $count ->
    [one] Buch
    [other] Bücher
}
books-finished = { $count ->
    [one] Buch abgeschlossen
    [other] Bücher abgeschlossen
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
notes = Notizen
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
    [other] { $count } Seiten
}
pages-label = Seiten

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
reading-sessions = Lese-Sitzungen
longest-session = Längste Sitzung
average-session = Ø Sitzung
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /Ø Sitzung
current-streak = Aktuelle Serie
longest-streak = Längste Serie
reading-streak = Lese-Serie
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
    [other] { $count } Tage
}
units-sessions = { $count ->
    [one] { $count } Sitzung
    [other] { $count } Sitzungen
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Mein KoShelf Lese-Rückblick
share = Teilen
share-recap-image = Rückblickbild teilen
download-recap-image = Rückblickbild herunterladen
download = Herunterladen
story = Story
story-aspect-ratio = 1260 × 2240 — Vertikal 9:16
square = Quadrat
square-aspect-ratio = 2160 × 2160 — Quadrat 1:1
banner = Banner
banner-aspect-ratio = 2400 × 1260 — Horizontal 2:1
best-month = Bester Monat
active-days = { $count ->
    [one] Aktiver Tag
    [other] Aktive Tage
}
hide = Ausblenden
show = Anzeigen
less = Weniger
more = Mehr
period = Zeitraum
sessions = Sitzungen
yearly-summary = Jahreszusammenfassung { $count }

# Navigation and sorting
aria-previous-year = Vorheriges Jahr
aria-next-year = Nächstes Jahr
sort-newest-first = Aktuell: Neueste zuerst
sort-oldest-first = Aktuell: Älteste zuerst
aria-toggle-sort-order = Sortierreihenfolge umschalten
aria-previous-month = Vorheriger Monat
aria-next-month = Nächster Monat
aria-search = Suchen
aria-close-search = Suche schließen
aria-close = Schließen

# -----------------------------------
#           Time & Dates
# -----------------------------------
month-january = Januar
month-february = Februar
month-march = März
month-april = April
month-may = Mai
month-june = Juni
month-july = Juli
month-august = August
month-september = September
month-october = Oktober
month-november = November
month-december = Dezember

# Abbreviated month names (for use in compact displays like heatmaps)
month-january-short = Jan
month-february-short = Feb
month-march-short = Mär
month-april-short = Apr
month-may-short = Mai
month-june-short = Jun
month-july-short = Jul
month-august-short = Aug
month-september-short = Sep
month-october-short = Okt
month-november-short = Nov
month-december-short = Dez

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday-mon = Mo
weekday-thu = Do
weekday-sun = So

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime-full-format = %-d. %B %Y um %H:%M Uhr
datetime-short-current-year-format = %-d. %b
datetime-short-with-year-format = %-d. %b %Y

# Time units: h=hours, m=minutes, d=days
units-h = h
units-m = min
units-d = d
today = Heute
of-the-year = des Jahres
of = von
last = Zuletzt

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] Tag
    [other] Tage
}
hours_label = { $count ->
    [one] Stunde
    [other] Stunden
}
minutes_label = { $count ->
    [one] Minute
    [other] Minuten
}