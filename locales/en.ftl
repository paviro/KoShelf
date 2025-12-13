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
comics = Comics
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
    .aria-label = Filter books
    .all = All
    .all-aria = { filter.aria-label } - Current: { filter.all }
    .reading = { status.reading }
    .reading-aria = { filter.aria-label } - Current: { filter.reading }
    .completed = { status.completed }
    .completed-aria = { filter.aria-label } - Current: { filter.completed }
    .unread = { status.unread }
    .unread-aria = { filter.aria-label } - Current: { filter.unread }
    .on-hold = { status.on-hold }
    .on-hold-aria = { filter.aria-label } - Current: { filter.on-hold }
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
comic-label = { $count ->
    [one] Comic
   *[other] Comics
}
books-finished = { $count ->
    [one] { book-label } Finished
   *[other] { book-label } Finished
}
comics-finished = { $count ->
    [one] { comic-label } Finished
   *[other] { comic-label } Finished
}
unknown-book = Unknown Book
unknown-author = Unknown Author
by = by
book-overview = Book Overview
comic-overview = Comic Overview

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
highlights-label = { $count ->
    [one] Highlight
   *[other] Highlights
}
notes = Notes
notes-label = { $count ->
    [one] Note
   *[other] Notes
}
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
pages-label = { $count ->
    [one] Page
   *[other] Pages
}

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
reading-sessions-label = { $count ->
    [one] Reading Session
   *[other] Reading Sessions
}
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
recap-story = Story
    .details = 1260 x 2240 — Vertical 9:16
recap-square = Square
    .details = 2160 x 2160 — Square 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
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
recap-empty =
    .nothing-here = Nothing here yet
    .try-switching = Try switching scope or year above.
    .finish-reading = Finish reading in KoReader to see your recap.
    .info-question = Why isn't my recap showing up?
    .info-answer = KoShelf uses reading statistics to detect book and comic completions, which allows tracking re-reads. Simply marking a book as "finished" without reading data will not make it appear here.
stats-empty =
    .nothing-here = Nothing here yet
    .start-reading = Start reading with KoReader to see your statistics here.
    .info-question = How does reading tracking work?
    .info-answer = KoReader automatically tracks your reading sessions, including time spent and pages read. Sync your statistics database to KoShelf to see your activity visualized here.

# Navigation and sorting
sort-order =
    .aria-label-toggle = Toggle sort order
    .newest-first = { sort-order.aria-label-toggle } - Current: Newest First
    .oldest-first = { sort-order.aria-label-toggle } - Current: Oldest First
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
go-back =
    .aria-label = Go back
select-month = Select Month

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