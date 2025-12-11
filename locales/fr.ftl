# ===========================================
#      French (fr) — Base Language File
# ===========================================
# This is the base translation file for French, using Metropolitan French (fr_FR).
# Regional variants (e.g., fr_CA.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., fr_CA.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = fr
-lang-name = Français
-lang-dialect = fr_FR

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Livres
statistics = Statistiques
calendar = Calendrier
recap = Récap
github = GitHub
loading = Chargement...
reload = Recharger
new-version-available = Nouvelle version disponible
tap-to-reload = Appuyez pour recharger
reading-companion = Compagnon de lecture
# Used in footer/sidebar for update time
last-updated = Dernière mise à jour
view-details = Voir les détails

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Rechercher livre, auteur, série...
filter-all = Tous
filter-reading = En cours
filter-completed = Terminé
filter-unread = Non lu
no-books-found = Aucun livre trouvé
no-books-match = Aucun livre ne correspond à votre recherche ou vos filtres.
try-adjusting = Essayez d’ajuster vos critères de recherche ou filtres
currently-reading = En cours de lecture
on-hold = En pause
# Also used as status
completed = Terminé
# Also used as status
unread = Non lu
book-label = { $count ->
    [one] Livre
   *[other] Livres
}
books-finished = { $count ->
    [one] Livre terminé
   *[other] Livres terminés
}
unknown-book = Livre inconnu
unknown-author = Auteur inconnu
by = par
book-overview = Aperçu du livre

# -----------------------------------
#            Book Details
# -----------------------------------
description = Description
publisher = Éditeur
series = Série
genres = Genres
language = Langue
book-identifiers = Identifiants du livre
my-review = Ma critique
my-note = Ma note
highlights = Surlignages
notes = Notes
bookmarks = Signets
page = Page
page-bookmark = Signet de page
bookmark-anchor = Ancre du signet
highlights-quotes = Surlignages & Citations
additional-information = Informations supplémentaires
reading-progress = Progression de lecture
page-number = Page { $count }
last-read = Dernière lecture
pages = { $count ->
    [one] { $count } page
   *[other] { $count } pages
}
pages-label = Pages

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Statistiques de lecture
overall-statistics = Statistiques globales
weekly-statistics = Statistiques hebdomadaires
total-read-time = Temps de lecture total
total-pages-read = Total des pages lues
pages-per-hour = Pages/heure
# Abbreviation for Pages Per Hour
pph-abbreviation = p/h
reading-sessions = Sessions de lecture
longest-session = Session la plus longue
average-session = Session moyenne
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /session moy.
current-streak = Série actuelle
longest-streak = Plus longue série
reading-streak = Série de lecture
days-read = Jours lus
weekly-reading-time = Temps de lecture hebdomadaire
weekly-pages-read = Pages lues cette semaine
average-time-day = Temps moyen/jour
average-pages-day = Pages moyennes/jour
most-pages-in-day = Plus grand nombre de pages lues en un jour
longest-daily-reading = Lecture quotidienne la plus longue
reading-completions = Lectures terminées
statistics-from-koreader = Statistiques des sessions KoReader
reading-time = Temps de lecture
pages-read = Pages lues
units-days = { $count ->
    [one] { $count } jour
   *[other] { $count } jours
}
units-sessions = { $count ->
    [one] { $count } session
   *[other] { $count } sessions
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = Mon récapitulatif de lecture KoShelf
share = Partager
share-recap-image = Partager l’image du récap
download-recap-image = Télécharger l’image du récap
download = Télécharger
story = Story
story-aspect-ratio = 1260 × 2240 — Vertical 9:16
square = Carré
square-aspect-ratio = 2160 × 2160 — Carré 1:1
banner = Bannière
banner-aspect-ratio = 2400 × 1260 — Horizontal 2:1
best-month = Meilleur mois
active-days = { $count ->
    [one] Jour actif
   *[other] Jours actifs
}
hide = Masquer
show = Afficher
less = Moins
more = Plus
period = Période
sessions = Sessions
yearly-summary = Résumé annuel { $count }

# Navigation and sorting
aria-previous-year = Année précédente
aria-next-year = Année suivante
sort-newest-first = Actuel : Plus récents d’abord
sort-oldest-first = Actuel : Plus anciens d’abord
aria-toggle-sort-order = Inverser l’ordre de tri
aria-previous-month = Mois précédent
aria-next-month = Mois suivant
aria-search = Rechercher
aria-close-search = Fermer la recherche
aria-close = Fermer

# -----------------------------------
#           Time & Dates
# -----------------------------------
month-january = Janvier
month-february = Février
month-march = Mars
month-april = Avril
month-may = Mai
month-june = Juin
month-july = Juillet
month-august = Août
month-september = Septembre
month-october = Octobre
month-november = Novembre
month-december = Décembre

# Abbreviated month names
month-january-short = Jan
month-february-short = Fév
month-march-short = Mar
month-april-short = Avr
month-may-short = Mai
month-june-short = Jun
month-july-short = Jul
month-august-short = Aoû
month-september-short = Sep
month-october-short = Oct
month-november-short = Nov
month-december-short = Déc

# Weekday abbreviations (Mon/Thu/Sun)
weekday-mon = Lun
weekday-thu = Jeu
weekday-sun = Dim

# Chrono date/time format strings
datetime-full-format = %-d %B %Y à %-H:%M
datetime-short-current-year-format = %-d %b
datetime-short-with-year-format = %-d %b %Y

# Time units
units-h = h
units-m = min
units-d = j
today = Aujourd’hui
of-the-year = de l’année
of = de
last = Dernier

# Time unit labels
days_label = { $count ->
    [one] jour
   *[other] jours
}
hours_label = { $count ->
    [one] heure
   *[other] heures
}
minutes_label = { $count ->
    [one] minute
   *[other] minutes
}
