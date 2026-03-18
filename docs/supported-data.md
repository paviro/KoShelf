# Supported Data

## Supported Formats

- ePUB
- fb2 / fb2.zip
- mobi (unencrypted)
- CBZ
- CBR (not supported on Windows — use the Linux build under [WSL](https://learn.microsoft.com/de-de/windows/wsl/install) if you need it)

## From EPUB Files

- Book title
- Authors
- Description (sanitized HTML)
- Cover image
- Language
- Publisher
- Series information (name and number)
- Identifiers (ISBN, ASIN, Goodreads, DOI, etc.)
- Subjects/Genres

## From FB2 Files

- Book title
- Authors
- Description (sanitized HTML)
- Cover image
- Language
- Publisher
- Series information (name and number)
- Identifiers (ISBN)
- Subjects/Genres

## From MOBI Files (unencrypted)

- Book title
- Authors
- Description
- Cover image
- Language
- Publisher
- Identifiers (ISBN, ASIN)
- Subjects/Genres

## From Comic Files (CBZ/CBR)

Note: **Windows builds support CBZ only** (CBR/RAR is not supported).

- Book title (from metadata or filename)
- Series information (Series and Number)
- Authors (writers, artists, editors, etc.)
- Description (Summary)
- Publisher
- Language
- Genres
- Cover image (first image in archive)

## From KoReader Metadata

- Reading status (reading/complete)
- Highlights and annotations with chapter information
- Notes attached to highlights
- Reading progress percentage
- Rating (stars out of 5)
- Summary note (the one you can fill out at the end of the book)
- Stable page metadata (`pagemap_*`) for stable page totals and optional synthetic page scaling (nightly / post-2025.10)

## From KoReader Statistics Database (statistics.sqlite3)

- Total reading time and pages
- Weekly reading statistics
- Reading activity heatmap with customizable scaling (automatic or fixed maximum)
- Per-book reading sessions and statistics
- Reading speed calculations
- Session duration tracking
- Book completions (used by Yearly Recap)
