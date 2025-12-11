# ===========================================
#      English (en) — Base Language File
# ===========================================
# This is the base translation file for English, using US English vocabulary.
# Regional variants (e.g., en_GB.ftl) should only override keys that differ.
#
# LOCALE HIERARCHY:
#   1. Regional variant (e.g., en_GB.ftl) — sparse, only differences
#   2. Base language (this file) — complete translations
#   3. English fallback (en.ftl) — ultimate fallback for all languages
#
# NEW KEYS: Always add new translation keys to en.ftl first, then other bases.

# Machine-readable metadata (used by --list-languages)
-lang-code = en
-lang-name = English
-lang-dialect = en_US

# -----------------------------------
#           Navigation & Shared
# -----------------------------------
books = Books
statistics = Statistics
calendar = Calendar
recap = Recap
github = GitHub
loading = Loading...
reload = Reload
new-version-available = New Version Available
tap-to-reload = Tap to reload
reading-companion = Reading Companion
# Used in footer/sidebar for update time
last-updated = Last updated
view-details = View Details

# -----------------------------------
#        Book List & Library
# -----------------------------------
search-placeholder = Search book, author, series...
filter-all = All
filter-reading = Reading
filter-completed = Completed
filter-unread = Unread
no-books-found = No Books Found
no-books-match = No books match your current search or filter criteria.
try-adjusting = Try adjusting your search or filter criteria
currently-reading = Currently Reading
on-hold = On Hold
# Also used as status
completed = Completed
# Also used as status
unread = Unread
book-label = { $count ->
    [one] Book
   *[other] Books
}
books-finished = { $count ->
    [one] Book Finished
   *[other] Books Finished
}
unknown-book = Unknown Book
unknown-author = Unknown Author
by = by
book-overview = Book Overview

# -----------------------------------
#            Book Details
# -----------------------------------
description = Description
publisher = Publisher
series = Series
genres = Genres
language = Language
book-identifiers = Book Identifiers
my-review = My Review
my-note = My Note
highlights = Highlights
notes = Notes
bookmarks = Bookmarks
page = Page
page-bookmark = Page Bookmark
bookmark-anchor = Bookmark anchor
highlights-quotes = Highlights & Quotes
additional-information = Additional Information
reading-progress = Reading Progress
page-number = Page { $count }
last-read = Last Read
pages = { $count ->
    [one] { $count } page
   *[other] { $count } pages
}
pages-label = Pages

# -----------------------------------
#       Statistics & Progress
# -----------------------------------
reading-statistics = Reading Statistics
overall-statistics = Overall Statistics
weekly-statistics = Weekly Statistics
total-read-time = Total Read Time
total-pages-read = Total Pages Read
pages-per-hour = Pages/Hour
# Abbreviation for Pages Per Hour
pph-abbreviation = pph
reading-sessions = Reading Sessions
longest-session = Longest Session
average-session = Average Session
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /avg session
current-streak = Current Streak
longest-streak = Longest Streak
reading-streak = Reading Streak
days-read = Days Read
weekly-reading-time = Weekly Reading Time
weekly-pages-read = Weekly Pages Read
average-time-day = Average Time/Day
average-pages-day = Average Pages/Day
most-pages-in-day = Most Pages in a Day
longest-daily-reading = Longest Daily Reading
reading-completions = Reading Completions
statistics-from-koreader = Statistics from KoReader reading sessions
reading-time = Reading Time
pages-read = Pages Read
units-days = { $count ->
    [one] { $count } day
   *[other] { $count } days
}
units-sessions = { $count ->
    [one] { $count } session
   *[other] { $count } sessions
}

# -----------------------------------
#               Recap
# -----------------------------------
my-reading-recap = My KoShelf Reading Recap
share = Share
share-recap-image = Share Recap Image
download-recap-image = Download Recap Image
download = Download
story = Story
story-aspect-ratio = 1260 × 2240 — Vertical 9:16
square = Square
square-aspect-ratio = 2160 × 2160 — Square 1:1
banner = Banner
banner-aspect-ratio = 2400 × 1260 — Horizontal 2:1
best-month = Best Month
active-days = { $count ->
    [one] Active Day
   *[other] Active Days
}
hide = Hide
show = Show
less = Less
more = More
period = Period
sessions = Sessions
yearly-summary = Yearly Summary { $count }

# Navigation and sorting
aria-previous-year = Previous year
aria-next-year = Next year
sort-newest-first = Current: Newest First
sort-oldest-first = Current: Oldest First
aria-toggle-sort-order = Toggle sort order
aria-previous-month = Previous month
aria-next-month = Next month
aria-search = Search
aria-close-search = Close search
aria-close = Close

# -----------------------------------
#           Time & Dates
# -----------------------------------
month-january = January
month-february = February
month-march = March
month-april = April
month-may = May
month-june = June
month-july = July
month-august = August
month-september = September
month-october = October
month-november = November
month-december = December

# Abbreviated month names (for use in compact displays like heatmaps)
month-january-short = Jan
month-february-short = Feb
month-march-short = Mar
month-april-short = Apr
month-may-short = May
month-june-short = Jun
month-july-short = Jul
month-august-short = Aug
month-september-short = Sep
month-october-short = Oct
month-november-short = Nov
month-december-short = Dec

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday-mon = Mon
weekday-thu = Thu
weekday-sun = Sun

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime-full-format = %B %-d, %Y at %-I:%M %p
datetime-short-current-year-format = %b %-d
datetime-short-with-year-format = %b %-d %Y

# Time units: h=hours, m=minutes, d=days
units-h = h
units-m = m
units-d = d
today = Today
of-the-year = of the year
of = of
last = Last

# Time unit labels (standalone word forms for displaying after numbers)
days_label = { $count ->
    [one] day
   *[other] days
}
hours_label = { $count ->
    [one] hour
   *[other] hours
}
minutes_label = { $count ->
    [one] minute
   *[other] minutes
}