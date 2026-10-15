# Supported Data

## Document Parsing

- ePUB
- fb2 / fb2.zip
- mobi (unencrypted)
- CBZ
- CBR (metadata and cover extraction requires a non-Windows build)

## Other Documents and Metadata Entries

KoShelf also scans PDF, DjVu (`djvu`/`djv`), XPS/OXPS, CBT/CB7, AZW/AZW3,
DOC/DOCX, RTF, ODT, CHM, PDB/PRC, FB3, and HTMLZ files. These documents can be
tracked and downloaded when present, but KoShelf does not extract their
metadata or covers from the document itself. CBR files have the same limitation
on Windows. CBZ, CBR, CBT, and CB7 are classified as comics.

A KOReader sidecar can introduce other extensions or extensionless documents.
New sidecar-only imports require a valid `partial_md5_checksum` and a
`doc_props` table. Custom metadata overrides take precedence; otherwise,
KoShelf uses previously extracted document metadata when available, or the
sidecar's document properties. Missing titles fall back to the saved document
filename, then the sidecar folder name.

Entries with only a sidecar show a **Metadata** badge. They support editing when
writeback is enabled, but have no document download or reader action. When both
sources disappear, [archive settings](configuration.md#library-preservation)
determine whether the saved entry remains visible. Unreachable roots preserve
the last known availability.

The in-app reader supports EPUB, FB2, MOBI, and CBZ. PDF and other unparsed
formats have no in-app rendering.

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

Note: Windows builds extract comic metadata and covers from CBZ only; CBR can still be tracked through KOReader metadata and downloaded when present.

- Book title (from metadata or filename)
- Series information (Series and Number)
- Authors (writers, artists, editors, etc.)
- Description (Summary)
- Publisher
- Language
- Genres
- Cover image (first image in archive)

## From KoReader Metadata

- Reading status (reading/complete/abandoned)
- Highlights and annotations with chapter information, including page-numbered PDF annotations
- PDF highlight geometry is preserved during edits; it is not rendered or exposed through the API
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
