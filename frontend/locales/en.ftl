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

# -----------------------------------
#             Settings
# -----------------------------------
appearance-setting = Appearance
theme-setting = Theme
    .description = Choose how KoShelf looks. Automatic follows your system preference.
    .option-auto = Automatic
    .option-light = Light
    .option-dark = Dark
prefetch-setting = Link prefetch
    .description = Preload pages when you hover, focus, or tap links to make navigation feel faster.
    .option-enabled = Enabled
    .option-disabled = Disabled
    .connection-note = Note: Prefetch is still skipped automatically when your connection is constrained (for example Data Saver or slower mobile networks).
language-setting = Language
    .hint = Sets the language used for all translations in the interface.
region-setting = Region
    .hint = Affects how dates and numbers are formatted.
    .preview-date = Preview Date
    .preview-number = Preview Number
    .majority-group = Majority-speaking regions
    .all-group = All Supported Regions

# -----------------------------------
#         Authentication
# -----------------------------------
login = Log In
    .title = Sign in to { $site }
    .password = Password
    .submit = Sign In
    .error = Invalid password
    .rate-limited = Too many attempts. Please try again shortly.
change-password = Change Password
    .setting = Password
    .current = Current Password
    .new = New Password
    .confirm = Confirm Password
    .changed = Password changed successfully
    .too-short = Password must be at least { $min } characters
    .mismatch = Passwords do not match
    .incorrect = Incorrect current password
    .current-placeholder = Enter current password
    .new-placeholder = Min. 8 characters
    .new-hint = Must be at least 8 characters
    .confirm-placeholder = Re-enter new password
session-management =
    .setting = Sessions
    .current = Current Session
    .this-device = This device
    .last-active = Last active
    .revoke = Revoke
    .revoke-confirm = Revoke this session?
    .revoked = Session revoked
    .device-info = { $browser } on { $os }
    .logout = Log Out

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
    .reading-short = Reading
    .on-hold = On Hold
    .completed = Completed
    .completed-short = Complete
    .unread = Unread
    .abandoned = Abandoned
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
statistics-from-koreader = Statistics from KoReader reading sessions.
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
    .go-to-list = Go to {$collection}
    .crash-title = Something went wrong
    .crash-description = An unexpected error occurred while rendering this page.
    .crash-report = Report issue

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

# -----------------------------------
#              Reader
# -----------------------------------
reader =
    .title = Reader
    .loading = Loading book...
    .previous-page = Previous page
    .next-page = Next page
    .contents = Contents
open-at-annotation = Open at annotation
open-in-reader = Open
    .aria-label = Open in reader
reader-settings =
    .label = Settings
    .aria-label = Display settings
    .typography = Typography
    .margins = Margins
reader-font-size = Font size
    .decrease-aria = Decrease font size
    .increase-aria = Increase font size
reader-line-spacing = Line spacing
    .decrease-aria = Decrease line spacing
    .increase-aria = Increase line spacing
reader-word-spacing = Word spacing
    .decrease-aria = Decrease word spacing
    .increase-aria = Increase word spacing
reader-hyphenation = Hyphenation
reader-floating-punctuation = Floating punctuation
reader-embedded-fonts = Embedded fonts
reader-left-margin = Left margin
    .decrease-aria = Decrease left margin
    .increase-aria = Increase left margin
reader-right-margin = Right margin
    .decrease-aria = Decrease right margin
    .increase-aria = Increase right margin
reader-top-margin = Top margin
    .decrease-aria = Decrease top margin
    .increase-aria = Increase top margin
reader-bottom-margin = Bottom margin
    .decrease-aria = Decrease bottom margin
    .increase-aria = Increase bottom margin
reader-mode =
    .auto = Book
    .on = On
    .off = Off
reader-reset =
    .book = Use book settings
    .book-aria = Reset to the book's synced display settings
    .defaults = Reset to defaults
    .defaults-aria = Reset reader display settings to defaults
reader-drawer =
    .label = Navigation
    .aria-label = Book navigation panel
reader-no-toc = No table of contents available
    .description = This file does not include chapter markers.
reader-no-highlights = No highlights yet
    .description = Highlights you add in KoReader will show up here.
reader-no-bookmarks = No bookmarks yet
    .description = Bookmarks you add in KoReader will show up here.
select-month = Select Month

# -----------------------------------
#           Time & Dates
# -----------------------------------
weekday =
    .mon = Mon
    .tue = Tue
    .wed = Wed
    .thu = Thu
    .fri = Fri
    .sat = Sat
    .sun = Sun

units =
    .w = w
    .d = d
    .h = h
    .m = m
    .s = s
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

# -----------------------------------
#         Writeback / Editing
# -----------------------------------
edit-warning =
    .title = Before you edit
    .body = Make sure the book is closed in KoReader and has been closed long enough for any file sync (e.g. Syncthing) to complete. KoReader overwrites the metadata file when a book is open. If you save changes here while the book is still open (or before the latest sidecar file has synced) your changes will be lost.
    .dismiss = Don't show this warning again
    .understood = Understood
edit =
    .aria-label = Edit
save = Save
cancel = Cancel
delete = Delete
no-review-available = No review available
    .hint-edit = Use the edit button to add your review and rating.
    .hint-readonly = Reviews can be added directly in KoReader.
add-review = Add review
delete-review = Delete review
add-note = Add note
edit-note = Edit note
delete-note = Delete note
delete-highlight = Delete highlight
delete-highlight-and-note = Delete highlight and note
delete-bookmark = Delete bookmark
change-status = Change Status
highlight-color =
    .aria-label = Highlight color
highlight-drawer =
    .aria-label = Highlight style
    .lighten = Highlight
    .underscore = Underline
    .strikeout = Strikeout
    .invert = Invert

# -----------------------------------
#         Toast notifications
# -----------------------------------
toast-dismiss-label = Dismiss
toast-update-item-error = Failed to save changes
    .subtitle = Your edits have been reverted.
toast-update-annotation-error = Failed to update annotation
    .subtitle = Your changes have been reverted.
toast-delete-annotation-error = Failed to delete annotation
    .subtitle = It has been restored.

# -----------------------------------
#            Page Activity
# -----------------------------------
page-activity = Page Activity
    .select-reading = Select reading
    .page = Page { $page }
    .unread = unread
    .visits = { $count ->
        [one] { $count } visit
       *[other] { $count } visits
    }
    .highlights = { $count ->
        [one] { $count } highlight
       *[other] { $count } highlights
    }
    .bookmarks = { $count ->
        [one] { $count } bookmark
       *[other] { $count } bookmarks
    }
    .notes = { $count ->
        [one] { $count } note
       *[other] { $count } notes
    }
    .of = of { $total }
    .pages-read = pages read
    .no-data = No page-level reading data available for this item.
    .legend-bookmark = Bookmark
    .legend-chapter = Chapter
    .info = Page numbers are affected by the current formatting options of the document. Unlike other parts of KoShelf, this view does not support synthetic page scaling.
