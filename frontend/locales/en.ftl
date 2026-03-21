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
settings = Settings
github = GitHub
reading-companion = Reading Companion
# Used in footer/sidebar for update time
last-updated = Last updated
view-details = View Details
appearance-setting = Appearance
theme-setting = Theme
theme-setting-description = Choose how KoShelf looks. Automatic follows your system preference.
theme-option-auto = Automatic
theme-option-light = Light
theme-option-dark = Dark
prefetch-setting = Link prefetch
prefetch-setting-description = Preload pages when you hover, focus, or tap links to make navigation feel faster.
prefetch-option-enabled = Enabled
prefetch-option-disabled = Disabled
prefetch-setting-connection-note = Note: Prefetch is still skipped automatically when your connection is constrained (for example Data Saver or slower mobile networks).
language-setting = Language
region-setting = Region
language-setting-hint = Sets the language used for all translations in the interface.
region-setting-hint = Affects how dates and numbers are formatted.
preview-date = Preview Date
preview-number = Preview Number
region-setting-majority-group = Majority-speaking regions
region-setting-all-group = All Supported Regions
login = Log In
login-title = Sign in to { $site }
login-password = Password
login-submit = Sign In
login-error = Invalid password
login-rate-limited = Too many attempts. Please try again shortly.
password-setting = Password
change-password = Change Password
current-password = Current Password
new-password = New Password
confirm-password = Confirm Password
password-changed = Password changed successfully
password-too-short = Password must be at least { $min } characters
password-mismatch = Passwords do not match
incorrect-password = Incorrect current password
sessions-setting = Sessions
current-session = Current Session
this-device = This device
last-active = Last active
revoke-session = Revoke
revoke-session-confirm = Revoke this session?
session-revoked = Session revoked
session-device-info = { $browser } on { $os }
logout = Log Out
current-password-placeholder = Enter current password
new-password-placeholder = Min. 8 characters
new-password-hint = Must be at least 8 characters
confirm-password-placeholder = Re-enter new password

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
notes-label = { $count ->
    [one] Note
   *[other] Notes
}
bookmarks = Bookmarks
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
yearly-statistics = Yearly Statistics
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
average-time-day = Average Time/Day
average-pages-day = Average Pages/Day
most-pages-in-day = Most Pages in a Day
longest-daily-reading = Longest Daily Reading
reading-completions = Reading Completions
completed-books = Completed Books
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
    .details = 1500 x 1500 — Square 1:1
recap-banner = Banner
    .details = 2400 x 1260 — Horizontal 2:1
best-month = Best Month
active-days = { $count ->
    [one] Active Day
   *[other] Active Days
}
active-days-tooltip = { $count ->
    [one] active day
   *[other] active days
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
error-state =
    .title = Something went wrong
    .description =
        The data could not be loaded.
        Please try again.
    .not-found-title = Not found
    .not-found-description = The page you're looking for doesn't exist or may have been removed.
    .connection-title = Connection failed
    .connection-description =
        Could not reach the server.
        Check your connection and try again.
    .file-unavailable-title = Book file unavailable
    .file-unavailable-description = The book details were found, but the book file is missing.
    .retry = Try again

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
go-back =
    .aria-label = Go back
open-reader-aria = Open in reader
reader-title = Reader
reader-loading = Loading book...
reader-previous-page = Previous page
reader-next-page = Next page
open-at-annotation = Open at annotation
reader-contents = Contents
reader-settings-aria = Display settings
reader-section-text = Text
reader-section-typography = Typography
reader-section-margins = Margins
reader-font-size = Font size
reader-font-size-decrease-aria = Decrease font size
reader-font-size-increase-aria = Increase font size
reader-line-spacing = Line spacing
reader-line-spacing-decrease-aria = Decrease line spacing
reader-line-spacing-increase-aria = Increase line spacing
reader-word-spacing = Word spacing
reader-word-spacing-decrease-aria = Decrease word spacing
reader-word-spacing-increase-aria = Increase word spacing
reader-hyphenation = Hyphenation
reader-floating-punctuation = Floating punctuation
reader-embedded-fonts = Embedded fonts
reader-left-margin = Left margin
reader-left-margin-decrease-aria = Decrease left margin
reader-left-margin-increase-aria = Increase left margin
reader-right-margin = Right margin
reader-right-margin-decrease-aria = Decrease right margin
reader-right-margin-increase-aria = Increase right margin
reader-top-margin = Top margin
reader-top-margin-decrease-aria = Decrease top margin
reader-top-margin-increase-aria = Increase top margin
reader-bottom-margin = Bottom margin
reader-bottom-margin-decrease-aria = Decrease bottom margin
reader-bottom-margin-increase-aria = Increase bottom margin
reader-mode-auto = Book
reader-mode-on = On
reader-mode-off = Off
reader-reset-book = Use book settings
reader-reset-book-aria = Reset to the book's synced display settings
reader-reset-defaults = Reset to defaults
reader-reset-defaults-aria = Reset reader display settings to defaults
reader-drawer-aria = Book navigation panel
reader-no-toc = No table of contents available
reader-no-toc-description = This file does not include chapter markers.
reader-no-highlights = No highlights yet
reader-no-highlights-description = Highlights you add in KoReader will show up here.
reader-no-bookmarks = No bookmarks yet
reader-no-bookmarks-description = Bookmarks you add in KoReader will show up here.
open-in-reader = Open
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
    .tue = Tue
    .wed = Wed
    .thu = Thu
    .fri = Fri
    .sat = Sat
    .sun = Sun

# Time units: w=weeks, d=days, h=hours, m=minutes
units =
    .w = w
    .d = d
    .h = h
    .m = m
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
