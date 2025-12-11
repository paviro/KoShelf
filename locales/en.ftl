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
-lang-name = English (United States)
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
filter =
    .all = All
    .reading = Reading
    .completed = Completed
    .unread = Unread
no-books-found = No Books Found
no-books-match = No books match your current search or filter criteria.
try-adjusting = Try adjusting your search or filter criteria
status =
    .reading = Currently Reading
    .on-hold = On Hold
    .completed = Completed
    .unread = Unread
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
session =
    .longest = Longest Session
    .average = Average Session
# Suffix for average session duration (e.g. '/avg session')
avg-session-suffix = /avg session
streak =
    .current = Current Streak
    .longest = Longest Streak
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
    .recap-label = Share Recap Image
download = Download
    .recap-label = Download Recap Image
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
toggle =
    .hide = Hide
    .show = Show
less = Less
more = More
period = Period
sessions = Sessions
yearly-summary = Yearly Summary { $count }

# Navigation and sorting
previous-year =
    .aria-label = Previous year
next-year =
    .aria-label = Next year
sort-order =
    .aria-label-toggle = Toggle sort order
sort-newest-first = Current: Newest First
sort-oldest-first = Current: Oldest First
previous-month =
    .aria-label = Previous month
next-month =
    .aria-label = Next month
search =
    .aria-label = Search
close-search =
    .aria-label = Close search
close = Close
    .aria-label = Close

# -----------------------------------
#           Time & Dates
# -----------------------------------
january = January
    .short = Jan
february = February
    .short = Feb
march = March
    .short = Mar
april = April
    .short = Apr
may = May
    .short = May
june = June
    .short = Jun
july = July
    .short = Jul
august = August
    .short = Aug
september = September
    .short = Sep
october = October
    .short = Oct
november = November
    .short = Nov
december = December
    .short = Dec

# Weekday abbreviations (only Mon/Thu/Sun for GitHub-style heatmap visualization)
weekday =
    .mon = Mon
    .thu = Thu
    .sun = Sun

# Chrono date/time format strings (use %B for full month, %b for short, etc.)
datetime =
    .full = %B %-d, %Y at %-I:%M %p
    .short-current-year = %b %-d
    .short-with-year = %b %-d %Y

# Time units: h=hours, m=minutes, d=days
units =
    .h = h
    .m = m
    .d = d
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