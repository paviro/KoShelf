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
-lang-name = Français (France)
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
filter =
    .aria-label = Filtrer les livres
    .all = Tous
    .all-aria = { filter.aria-label } - Actuel : { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Actuel : { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Actuel : { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Actuel : { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Actuel : { filter.on-hold }
no-books-found = Aucun livre trouvé
no-books-match = Aucun livre ne correspond à votre recherche ou vos filtres.
try-adjusting = Essayez d’ajuster vos critères de recherche ou filtres
status =
    .reading = En cours de lecture
    .on-hold = En pause
    .completed = Terminé
    .unread = Non lu
book-label = { $count ->
    [one] Livre
   *[other] Livres
}
books-finished = { $count ->
    [one] { book-label } terminé
   *[other] { book-label } terminés
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
session =
    .longest = Session la plus longue
    .average = Session moyenne
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /session moy.
streak =
    .current = Série actuelle
    .longest = Plus longue série
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
    .recap-label = Partager l'image du récap
download = Télécharger
    .recap-label = Télécharger l’image du récap
recap-story = Story
    .details = 1260 x 2240 — Vertical 9:16
recap-square = Carré
    .details = 2160 x 2160 — Carré 1:1
recap-banner = Bannière
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Meilleur mois
active-days = { $count ->
    [one] Jour actif
   *[other] Jours actifs
}
toggle =
    .hide = Masquer
    .show = Afficher
less = Moins
more = Plus
period = Période
sessions = Sessions
yearly-summary = Résumé annuel { $count }

# Navigation and sorting
previous-year =
    .aria-label = Année précédente
next-year =
    .aria-label = Année suivante
sort-order = 
    .aria-label-toggle = Inverser l’ordre de tri
    .newest-first = { sort-order.aria-label-toggle } - Actuel : Plus récents d’abord
    .oldest-first = { sort-order.aria-label-toggle } - Actuel : Plus anciens d’abord
previous-month = 
    .aria-label = Mois précédent
next-month = 
    .aria-label = Mois suivant
search = 
    .aria-label = Rechercher
close-search = 
    .aria-label = Fermer la recherche
close = Fermer
    .aria-label = Fermer
go-back =
    .aria-label = Retour

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = Janvier
    .short = Jan
february = Février
    .short = Fév
march = Mars
    .short = Mar
april = Avril
    .short = Avr
may = Mai
    .short = Mai
june = Juin
    .short = Jun
july = Juillet
    .short = Jul
august = Août
    .short = Aoû
september = Septembre
    .short = Sep
october = Octobre
    .short = Oct
november = Novembre
    .short = Nov
december = Décembre
    .short = Déc

# Weekday abbreviations (Mon/Thu/Sun)
weekday =
    .mon = Lun
    .thu = Jeu
    .sun = Dim

# Chrono date/time format strings
datetime =
    .full = %-d %B %Y à %-H:%M
    .short-current-year = %-d %b
    .short-with-year = %-d %b %Y

# Time units
units =
    .h = h
    .m = min
    .d = j
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
