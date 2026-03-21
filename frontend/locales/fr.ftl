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
comics = Bandes dessinées
statistics = Statistiques
calendar = Calendrier
recap = Récap
settings = Paramètres
github = GitHub
reading-companion = Compagnon de lecture
# Used in footer/sidebar for update time
last-updated = Dernière mise à jour
view-details = Voir les détails

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Apparence
theme-setting = Thème
    .description = Choisissez l'apparence de KoShelf. Le mode automatique suit votre préférence système.
    .option-auto = Automatique
    .option-light = Clair
    .option-dark = Sombre
prefetch-setting = Préchargement des liens
    .description = Préchargez des pages lorsque vous survolez, focalisez ou touchez des liens pour rendre la navigation plus rapide.
    .option-enabled = Activé
    .option-disabled = Désactivé
    .connection-note = Remarque : le préchargement est toujours ignoré automatiquement lorsque votre connexion est limitée (par exemple Économiseur de données ou réseaux mobiles plus lents).
language-setting = Langue
    .hint = Définit la langue utilisée pour toutes les traductions de l'interface.
region-setting = Région
    .hint = Affecte le format des dates et des nombres.
    .preview-date = Aperçu Date
    .preview-number = Aperçu Nombre
    .majority-group = Régions à majorité linguistique
    .all-group = Toutes les régions prises en charge

# -----------------------------------
#         Authentication
# -----------------------------------
login = Se connecter
    .title = Connectez-vous à { $site }
    .password = Mot de passe
    .submit = Se connecter
    .error = Mot de passe invalide
    .rate-limited = Trop de tentatives. Veuillez réessayer bientôt.
change-password = Changer le mot de passe
    .setting = Mot de passe
    .current = Mot de passe actuel
    .new = Nouveau mot de passe
    .confirm = Confirmer le mot de passe
    .changed = Mot de passe modifié avec succès
    .too-short = Le mot de passe doit comporter au moins { $min } caractères
    .mismatch = Les mots de passe ne correspondent pas
    .incorrect = Mot de passe actuel incorrect
    .current-placeholder = Saisir le mot de passe actuel
    .new-placeholder = Min. 8 caractères
    .new-hint = Doit comporter au moins 8 caractères
    .confirm-placeholder = Ressaisir le nouveau mot de passe
session-management =
    .setting = Sessions
    .current = Session actuelle
    .this-device = Cet appareil
    .last-active = Dernière activité
    .revoke = Révoquer
    .revoke-confirm = Révoquer cette session ?
    .revoked = Session révoquée
    .device-info = { $browser } sur { $os }
    .logout = Se déconnecter

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
comic-label = { $count ->
    [one] Bande dessinée
   *[other] Bandes dessinées
}
books-finished = { $count ->
    [one] { book-label } terminé
   *[other] { book-label } terminés
}
comics-finished = { $count ->
    [one] { comic-label } terminée
   *[other] { comic-label } terminées
}
unknown-book = Livre inconnu
unknown-author = Auteur inconnu
by = par
book-overview = Aperçu du livre
comic-overview = Aperçu de la bande dessinée

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
highlights-label = { $count ->
    [one] Surlignage
   *[other] Surlignages
}
notes-label = { $count ->
    [one] Note
   *[other] Notes
}
bookmarks = Signets
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
pages-label = { $count ->
    [one] Page
   *[other] Pages
}

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Statistiques de lecture
overall-statistics = Statistiques globales
weekly-statistics = Statistiques hebdomadaires
yearly-statistics = Statistiques annuelles
total-read-time = Temps de lecture total
total-pages-read = Total des pages lues
pages-per-hour = Pages/heure
# Abbreviation for Pages Per Hour
pph-abbreviation = p/h
reading-sessions-label = { $count ->
    [one] Session de lecture
   *[other] Sessions de lecture
}
session =
    .longest = Session la plus longue
    .average = Session moyenne
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /session moy.
streak =
    .current = Série actuelle
    .longest = Plus longue série
reading-streak = Série de lecture
average-time-day = Temps moyen/jour
average-pages-day = Pages moyennes/jour
most-pages-in-day = Plus grand nombre de pages lues en un jour
longest-daily-reading = Lecture quotidienne la plus longue
reading-completions = Lectures terminées
completed-books = Livres terminés
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
    .details = 1500 x 1500 — Carré 1:1
recap-banner = Bannière
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Meilleur mois
active-days = { $count ->
    [one] Jour actif
   *[other] Jours actifs
}
active-days-tooltip = { $count ->
    [one] jour actif
   *[other] jours actifs
}
toggle =
    .hide = Masquer
    .show = Afficher
less = Moins
more = Plus
period = Période
sessions = Sessions
yearly-summary = Résumé annuel { $count }
recap-empty =
    .nothing-here = Il n’y a encore rien ici
    .try-switching = Essayez de changer la période ou l'année au dessus.
    .finish-reading = Terminez un livre dans KoReader pour voir votre récapitulatif.
    .info-question = Pourquoi mon récapitulatif ne s’affiche-t-il pas ?
    .info-answer = KoShelf utilise les statistiques de lecture pour détecter les livres et bandes dessinées terminés, ce qui permet de suivre les relectures. Marquer simplement un livre comme "terminé" sans données de lecture ne le fera pas apparaître ici.

stats-empty =
    .nothing-here = Il n’y a encore rien ici
    .start-reading = Commencez à lire avec KoReader pour voir vos statistiques ici.
    .info-question = Comment fonctionne le suivi de lecture ?
    .info-answer = KoReader enregistre automatiquement vos sessions de lecture, y compris le temps passé et les pages lues. Synchronisez votre base de données de statistiques avec KoShelf pour visualiser vos activités ici.
error-state =
    .title = Une erreur est survenue
    .description =
        Les données n'ont pas pu être chargées.
        Veuillez réessayer.
    .not-found-title = Introuvable
    .not-found-description = La page que vous recherchez n'existe pas ou a été supprimée.
    .connection-title = Connexion échouée
    .connection-description =
        Impossible de joindre le serveur.
        Vérifiez votre connexion et réessayez.
    .file-unavailable-title = Fichier du livre indisponible
    .file-unavailable-description = Les détails du livre ont été trouvés, mais le fichier du livre est manquant.
    .retry = Réessayer

# Navigation and sorting
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
go-back =
    .aria-label = Retour

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Lecteur
    .loading = Chargement du livre…
    .previous-page = Page précédente
    .next-page = Page suivante
    .contents = Sommaire
open-at-annotation = Ouvrir à l’annotation
open-in-reader = Ouvrir
    .aria-label = Ouvrir dans le lecteur
reader-settings =
    .aria-label = Paramètres d’affichage
    .typography = Typographie
    .margins = Marges
reader-font-size = Taille de police
    .decrease-aria = Diminuer la taille de police
    .increase-aria = Augmenter la taille de police
reader-line-spacing = Interligne
    .decrease-aria = Diminuer l’interligne
    .increase-aria = Augmenter l’interligne
reader-word-spacing = Espacement des mots
    .decrease-aria = Diminuer l’espacement des mots
    .increase-aria = Augmenter l’espacement des mots
reader-hyphenation = Césure
reader-floating-punctuation = Ponctuation suspendue
reader-embedded-fonts = Polices intégrées
reader-left-margin = Marge gauche
    .decrease-aria = Réduire la marge gauche
    .increase-aria = Augmenter la marge gauche
reader-right-margin = Marge droite
    .decrease-aria = Réduire la marge droite
    .increase-aria = Augmenter la marge droite
reader-top-margin = Marge supérieure
    .decrease-aria = Réduire la marge supérieure
    .increase-aria = Augmenter la marge supérieure
reader-bottom-margin = Marge inférieure
    .decrease-aria = Réduire la marge inférieure
    .increase-aria = Augmenter la marge inférieure
reader-mode =
    .auto = Livre
    .on = Activé
    .off = Désactivé
reader-reset =
    .book = Utiliser les paramètres du livre
    .book-aria = Rétablir les paramètres d’affichage synchronisés du livre
    .defaults = Réinitialiser par défaut
    .defaults-aria = Réinitialiser les paramètres d’affichage du lecteur par défaut
reader-drawer =
    .aria-label = Panneau de navigation du livre
reader-no-toc = Aucun sommaire disponible
    .description = Ce fichier n’inclut pas de repères de chapitre.
reader-no-highlights = Aucun surlignage pour le moment
    .description = Les surlignages que vous ajoutez dans KoReader s’afficheront ici.
reader-no-bookmarks = Aucun signet pour le moment
    .description = Les signets que vous ajoutez dans KoReader s’afficheront ici.
select-month = Sélectionner le mois

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
    .tue = Mar
    .wed = Mer
    .thu = Jeu
    .fri = Ven
    .sat = Sam
    .sun = Dim

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = sem
    .d = j
    .h = h
    .m = min
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
