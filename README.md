<div align="center">

# KoShelf

<p>
  <a href="https://github.com/paviro/koshelf/stargazers">
    <img src="https://img.shields.io/github/stars/paviro/koshelf?style=social" alt="Stars" />
  </a>
  <a href="https://github.com/paviro/koshelf/releases/latest">
    <img src="https://img.shields.io/github/v/release/paviro/koshelf?label=release" alt="Latest Release" />
  </a>
  <a href="https://github.com/paviro/koshelf/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/paviro/koshelf" alt="License" />
  </a>
</p>

![Statistics dashboard](https://github.com/user-attachments/assets/94a094d2-298b-412c-80b3-b3b2e2cfc6de)

###### A Rust CLI tool that generates a beautiful website from your KoReader library, showcasing your ebook collection with highlights, annotations, and reading progress.

</div>

## Table of Contents

- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
    - [Home Assistant](#home-assistant)
    - [Prebuilt Binaries](#prebuilt-binaries)
    - [From Source](#from-source)
- [Usage](#usage)
    - [Basic Usage](#basic-usage)
    - [Operation Modes](#operation-modes)
    - [Command Line Options](#command-line-options)
    - [Subcommands](#subcommands)
    - [Configuration Sources & Precedence](#configuration-sources--precedence)
    - [Authentication in Serve Mode](#authentication-in-serve-mode)
    - [Stable Page Metadata & Scaling](#stable-page-metadata--scaling)
    - [Example](#example)
- [KoReader Setup](#koreader-setup)
    - [Metadata Storage Options](#metadata-storage-options)
    - [Typical Deployment Setup](#typical-deployment-setup)
- [Supported Data](#supported-data)
    - [From EPUB Files](#from-epub-files)
    - [From KoReader Metadata](#from-koreader-metadata)
    - [From KoReader Statistics Database](#from-koreader-statistics-database-statisticssqlite3)
- [API Shape](#api-shape)
- [Generated Site Structure](#generated-site-structure)
- [Credits](#credits)
- [Disclaimer](#disclaimer)

## Features

- 📚 **Library Overview (Books + Comics)**: Displays your currently reading, completed, and unread items (ebooks + comics)
- 🎨 **Modern UI**: Beautiful design powered by Tailwind CSS with clean typography and responsive layout
- 📝 **Annotations, Highlights & Ratings**: All your KoReader highlights, notes, star ratings, and review notes (summary note) are shown together on each book's details page with elegant formatting
- 📊 **Reading Statistics**: Track your reading habits with detailed statistics including reading time, pages read, customizable activity heatmaps, and weekly breakdowns
- 📅 **Reading Calendar**: Monthly calendar view showing your reading activity with items read on each day and monthly statistics
- 🎉 **Yearly Recap**: Celebrate your reading year with a timeline of completions, monthly summaries (finished items, hours read), and rich per-item details
- 📈 **Per-Item Statistics**: Detailed statistics for each item including session count, average session duration, reading speed, and last read date
- 🔍 **Search & Filter**: Search through your library by title, author, or series, with filters for reading status
- 🔐 **Optional Authentication**: Password-protect server mode with session-based auth and login rate limiting
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 🖥️ **Server Mode**: Built-in web server with live file watching for use with reverse proxy
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts

## Screenshots

![Library overview](https://github.com/user-attachments/assets/ad096bc9-c53a-40eb-9de9-06085e854a26)
![Book details](https://github.com/user-attachments/assets/44113be0-aa19-4018-b864-135ddb067a9d)
![Reading calendar](https://github.com/user-attachments/assets/a4ac51f1-927e-463d-b2d6-72c29fdc4323)
![Recap](https://github.com/user-attachments/assets/9558eea9-dee1-4b0a-adac-1bc0157f0181)

## Installation

### Home Assistant

Using Home Assistant? Install KoShelf as an add-on with just one click below.

[![Open your Home Assistant instance and show the dashboard of an add-on.](https://my.home-assistant.io/badges/supervisor_addon.svg)](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

### Docker Compose Deployment

Deploy KoShelf easily using the community-maintained Docker image.

#### Quick Start

1. Create a docker-compose.yml file:

```yaml
services:
    koshelf:
        image: ghcr.io/devtigro/koshelf:latest
        ports:
            - '3000:3000'
        volumes:
            - /path/to/your/books:/books:ro
            - /path/to/your/settings:/settings:ro
        restart: unless-stopped
```

2. Update the volume paths:

- Replace `/path/to/your/books` with the absolute path to your book library
- Replace `/path/to/your/settings` with the absolute path to your settings directory

3. Start the container:

```bash
docker compose up -d
```

4. Access KoShelf at http://localhost:3000

Docker Image Repository: [koshelf-docker](https://github.com/DevTigro/koshelf-docker)

### Prebuilt Binaries

The easiest way to get started is to download a prebuilt binary from the [releases page](https://github.com/paviro/koshelf/releases). Binaries are available for:

- Windows (x64)
- macOS (Apple Silicon, Intel & Universal)
- Linux (x64 and ARM64)

Please note that KoShelf is a command line tool, so you will need to execute it from within a terminal (macOS/Linux) or PowerShell/Command Prompt on Windows. Simply double-clicking the executable won't work since it requires command line arguments to function properly.

**Note for Windows users**: Windows Defender will likely flag and delete the Windows binary as a virus (more information [here](https://medium.com/opsops/is-windows-broken-7f8de8b8f3ad)). This is a false positive if you downloaded the binary directly from this repo. To use the binary:

1. Restore it from Windows Defender's protection history (Windows Security > Virus & threat protection > Protection history > Restore)
2. Launch the binary from PowerShell or Windows Terminal with arguments - double-clicking will cause it to close immediately since no arguments are provided

#### First Time Using Command Line?

If you've never used a command line before, here's how to get started:

**Windows:**

1. Press `Win + R`, type `powershell`, and press Enter
2. Navigate to where you downloaded the KoShelf binary (e.g., `cd C:\Users\YourName\Downloads`)
3. Run the tool with your desired arguments (see examples below)

**macOS and Linux:**

1. Press `Cmd + Space`, type `terminal`, and press Enter
2. Navigate to where you downloaded the KoShelf binary (e.g., `cd ~/Downloads`)
3. Make the file executable: `chmod +x koshelf` (should not be needed on macOS as the binary is signed)
4. Run the tool with your desired arguments (see examples below)

**Example:**

```bash
# Navigate to your downloads folder
cd ~/Downloads  # macOS/Linux
cd C:\Users\YourName\Downloads  # Windows

# Run KoShelf with your books folder
./koshelf --library-path /path/to/your/library --output ./my-library-site
```

**Pro tip:** On most terminals, you can drag and drop the downloaded binary file directly into the terminal window. This will automatically insert the full file path, allowing you to immediately add your arguments and run the command.

If you plan to use KoShelf frequently and use Linux or macOS, you can move the binary to `/usr/local/bin/` to make it available system-wide. This allows you to run `koshelf` from anywhere without specifying the full path:

```bash
# Move the binary to system PATH (requires sudo)
sudo mv koshelf /usr/local/bin/

# Now you can run it from anywhere
koshelf --library-path ~/Books --output ~/my-library-site
```

### From Source

If you prefer to build from source or need a custom build:

#### Prerequisites

- Rust 1.70+ (for building)
- Node.js and npm (React frontend build pipeline)

#### Building the tool

```bash
git clone https://github.com/paviro/KoShelf
cd KoShelf

# Build the Rust binary
cargo build --release
```

The binary will be available at `target/release/koshelf`.

**Note:** The React frontend is built during `cargo build` and embedded into the binary.

## Usage

### Basic Usage

```bash
./koshelf --library-path /path/to/your/library --output ./my-library-site
```

### Operation Modes

KoShelf can operate in several modes:

1. **Static Site Generation**: Generate a static site once and exit (default when `--output` is specified without `--watch`)
2. **Web Server Mode**: Serves the embedded React app at `/` with API endpoints under `/api/**`, and automatically refreshes data on library changes (default when `--output` is not specified). Requires `--data-path`.
3. **Watch Mode**: Generate a static site, rebuilding when book files change (when both `--output` and `--watch` are specified)

### Command Line Options

- `-c, --config`: Path to a TOML configuration file (`koshelf.toml` is auto-loaded when present)
- `-i, --library-path`: Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata. Can be specified multiple times. (optional if `--statistics-db` is provided)
- `--docsettings-path`: Path to KOReader's `docsettings` folder for users who store metadata separately (requires `--library-path`, mutually exclusive with `--hashdocsettings-path`)
- `--hashdocsettings-path`: Path to KOReader's `hashdocsettings` folder for users who store metadata by content hash (requires `--library-path`, mutually exclusive with `--docsettings-path`)
- `-s, --statistics-db`: Path to the `statistics.sqlite3` file for additional reading stats (optional if `--library-path` is provided)
- `-o, --output`: Output directory for the generated site
- `--data-path`: Persistent runtime data directory for serve mode (required when `--output` is not set)
- `-p, --port`: Port for web server mode (default: 3000)
- `-w, --watch`: Enable file watching with static output (requires `--output`)
- `-t, --title`: Site title (default: "KoShelf")
- `--enable-auth`: Enable password authentication in serve mode
- `--trusted-proxies`: Comma-separated or repeated trusted reverse proxy IP/CIDR entries used for forwarded client IP/proto resolution
- `--include-unread`: Include unread items (files without KoReader metadata)
- `--heatmap-scale-max`: Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is `2h` (pass `auto` for automatic scaling)
- `--timezone`: Timezone to interpret timestamps (IANA name, e.g., `Australia/Sydney`); defaults to system local
- `--day-start-time`: Logical day start time as `HH:MM` (default: `00:00`)
- `--min-pages-per-day`: Minimum pages read per book per day to be counted in statistics (optional)
- `--min-time-per-day`: Minimum reading time per book per day to be counted in statistics (e.g., "30s", "15m", "1h", `off`). Default is `30s`.
    > **Note:** If both `--min-pages-per-day` and `--min-time-per-day` are provided, a book's data for a day is counted if **either** condition is met for that book on that day. These filters apply **per book per day**, meaning each book must individually meet the threshold for each day to be included in statistics. Since `--min-time-per-day` defaults to `30s`, it is active unless explicitly overridden. Use `--min-time-per-day off` to disable this filter.
- `--include-all-stats`: By default, statistics are filtered to only include books present in your `--library-path` directories. This prevents deleted books or external files (like Wallabag articles) from skewing your recap and statistics. Use this flag to include statistics for all books in the database, regardless of whether they exist in your library.
- `--ignore-stable-page-metadata`: Ignore KOReader stable page metadata for page totals and page-based stats scaling. By default, stable metadata is used when available.
- `-l, --language`: Default server language for UI translations. Frontend language/region settings can override this per browser. Use full locale code (e.g., `en_US`, `de_DE`, `pt_BR`) for correct date formatting. Default: `en_US` (see `list-languages` subcommand)

### Subcommands

- `koshelf list-languages`: Print all supported UI locales
- `koshelf github`: Print the repository URL
- `koshelf set-password --data-path <PATH> [--password <VALUE>] [--overwrite]`: Set or rotate the serve-mode authentication password
    - without `--password`, KoShelf prompts interactively
    - `--data-path` can be omitted when provided by `KOSHELF_DATA_PATH` or `koshelf.toml`
    - without `--overwrite`, command is idempotent (no-op if password already exists)

### Configuration Sources & Precedence

For main app runs (non-subcommand flow), settings are merged in this order:

1. CLI arguments
2. Environment variables
3. Config file (`--config` or default `koshelf.toml`)
4. Built-in defaults

Common environment variables:

- `KOSHELF_LIBRARY_PATH`
- `KOSHELF_STATISTICS_DB`
- `KOSHELF_OUTPUT`
- `KOSHELF_DATA_PATH`
- `KOSHELF_ENABLE_AUTH`
- `KOSHELF_TRUSTED_PROXIES`
- `KOSHELF_TITLE`
- `KOSHELF_LANGUAGE`

Use `koshelf --help` to see the full env mapping for every option.

Subcommands use the same precedence model. For `set-password`, data path resolution is: command `--data-path` > top-level `--data-path` / `KOSHELF_DATA_PATH` > config file.

### Authentication in Serve Mode

Authentication is optional and only applies in serve mode.

1. Start server with a persistent data path: `--data-path /path/to/runtime-data`
2. Enable auth: `--enable-auth`
3. On first run, KoShelf generates a password and prints it once
4. Rotate password anytime via `koshelf set-password --data-path /path/to/runtime-data --overwrite`

Protected routes include `/api/**` (except `GET /api/site` and `POST /api/auth/login`) and runtime assets under `/assets/**`. Shell assets under `/core/**` remain public.

### Stable Page Metadata & Scaling

KoShelf can use KOReader stable page metadata to improve page totals and page-based stats.

- **Stable total pages for display** are used when KOReader metadata contains:
  - `pagemap_doc_pages > 0`
- This display behavior works for both publisher labels and synthetic mode.
- **Synthetic page scaling for statistics** is applied only when synthetic metadata is also present:
  - `pagemap_doc_pages > 0`
  - `pagemap_chars_per_synthetic_page`
- If you use publisher labels without synthetic override, KoShelf still shows stable total pages, but page-based statistics stay unscaled.
- Why publisher-label mode stays unscaled: KoShelf rescales stats using one linear factor (`stable_total / rendered_total`) across page events. That works for KOReader synthetic pagination (uniform char-based pages), but publisher labels are often non-linear (front matter, skipped/duplicate labels, appendix jumps). Applying one factor there would distort pages/day and pages/hour.
- If these `pagemap_*` fields are missing, KoShelf uses KOReader's normal `doc_pages`/statistics values and does not apply synthetic scaling.

For consistent page-based comparisons between books, enable KOReader's `Override publisher page numbers` setting. This makes KOReader persist synthetic metadata, which lets KoShelf rescale page metrics across books.

Compatibility note:

- This feature requires KOReader nightly builds or a future stable release after `2025.10 "Ghost"`.
- KOReader `2025.10 "Ghost"` does not write the required `pagemap_*` metadata fields, so KoShelf uses its standard unscaled page behavior.
- After updating to a KOReader build newer than `2025.10 "Ghost"`, you can use [KoReader-PopulateStablePageKOReader](https://github.com/paviro/KoReader-PopulateStablePageKOReader) to backfill stable page metadata for already finished books.

### Example

```bash
# Generate site from a library folder
./koshelf -i ~/Library -o ~/my-reading-site -t "My Reading Journey"

# Generate site from multiple folders (e.g., books + comics)
./koshelf -i ~/Books -i ~/Comics -o ~/my-reading-site

# Generate site with statistics and unread items included
./koshelf -i ~/Library -o ~/my-reading-site --statistics-db ~/KOReaderSettings/statistics.sqlite3 --include-unread

# Start web server with live file watching and statistics
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --data-path ~/koshelf-data -p 8080

# Start web server with authentication enabled (password generated on first run)
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --data-path ~/koshelf-data --enable-auth

# Set/rotate auth password explicitly
./koshelf set-password --data-path ~/koshelf-data --overwrite

# Generate static site with file watching and statistics
./koshelf --library-path ~/Library -o ~/my-reading-site --statistics-db ~/KOReaderSettings/statistics.sqlite3 --watch

# Generate site with custom heatmap color scaling (3 hours = highest intensity)
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --heatmap-scale-max 3h

# Generate site with custom heatmap color scaling (1.5 hours = highest intensity)
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --heatmap-scale-max 1h30m

# Generate site with explicit timezone and non-midnight day start (good for night owls)
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --timezone Australia/Sydney --day-start-time 03:00

# Using hashdocsettings (metadata stored by content hash)
./koshelf -i ~/Books -o ~/my-reading-site --hashdocsettings-path ~/KOReaderSettings/hashdocsettings

# Using docsettings (metadata stored in central folder by path)
./koshelf -i ~/Books -o ~/my-reading-site --docsettings-path ~/KOReaderSettings/docsettings

# Generate site with German UI language
./koshelf -i ~/Library -o ~/my-reading-site --language de_DE

# Ignore stable metadata page totals and synthetic scaling
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --ignore-stable-page-metadata
```

## KoReader Setup

### Metadata Storage Options

KOReader offers three ways to store book metadata (reading progress, highlights, annotations). KOShelf supports all three:

#### 1. Default: Metadata Next to Books (Recommended)

By default, KOReader creates `.sdr` folders next to each book file:

```
Books/
├── Book Title.epub
├── Book Title.sdr/
│   └── metadata.epub.lua
├── Another Book.epub
├── Another Book.sdr/
│   └── metadata.epub.lua
└── ...
```

This is the simplest setup - just point `--library-path` to your books folder.

#### 2. Hashdocsettings

If you select "hashdocsettings" in KOReader settings, metadata is stored in a central folder organized by content hash:

```
KOReaderSettings/
└── hashdocsettings/
    ├── 57/
    │   └── 570615f811d504e628db1ef262bea270.sdr/
    │       └── metadata.epub.lua
    └── a3/
        └── a3b2c1d4e5f6...sdr/
            └── metadata.epub.lua
```

**Usage:**

```bash
./koshelf --library-path ~/Books --hashdocsettings-path ~/KOReaderSettings/hashdocsettings
```

#### 3. Docsettings

If you select "docsettings" in KOReader settings, KOReader mirrors your book folder structure in a central folder and stores the metadata there:

```
KOReaderSettings/
└── docsettings/
    └── home/
        └── user/
            └── Books/
                ├── Book Title.sdr/
                │   └── metadata.epub.lua
                └── Another Book.sdr/
                    └── metadata.epub.lua
```

**Note:** Unlike KOReader, KOShelf matches books by filename only, since the folder structure reflects the device path (which may differ from your local path). If you have multiple books with the same filename, KOShelf will show an error - use `hashdocsettings` or `book folder` instead.

**Usage:**

```bash
./koshelf --library-path ~/Books --docsettings-path ~/KOReaderSettings/docsettings
```

### Typical Deployment Setup

Although there are many ways to use this tool here is how I use it:

1. **Syncthing Sync**: I use [Syncthing](https://syncthing.net/) to sync both my books folder and KoReader settings folder from my e-reader to my server
2. **Books and Statistics**: I point to the synced books folder with `--library-path` and to `statistics.sqlite3` in the synced KoReader settings folder with `--statistics-db`
3. **Web Server Mode**: I then run KoShelf in web server mode (without `--output`) - it will automatically rebuild when files change
4. **Nginx Reverse Proxy**: I use an nginx reverse proxy for HTTPS and to restrict access

My actual setup:

```bash
# My server command - runs continuously with file watching and statistics
./koshelf --library-path ~/syncthing/Books \
         --statistics-db ~/syncthing/KOReaderSettings/statistics.sqlite3 \
         --port 3000
```

This way, every time Syncthing pulls updates from my e-reader, the website automatically updates with my latest reading progress, new highlights, and updated statistics.

### User-Contributed Setups

See [Syncthing Setups](docs/syncthing_setups/README.md) for community-contributed guides on how to sync your devices with KoShelf.

## Supported Data

### Supported Formats

- ePUB
- fb2 / fb2.zip
- mobi (unencrypted)
- CBZ
- CBR (not supported on Windows - use the linux build under [WSL](https://learn.microsoft.com/de-de/windows/wsl/install) if you need it)

### From EPUB Files

- Book title
- Authors
- Description (sanitized HTML)
- Cover image
- Language
- Publisher
- Series information (name and number)
- Identifiers (ISBN, ASIN, Goodreads, DOI, etc.)
- Subjects/Genres

### From FB2 Files

- Book title
- Authors
- Description (sanitized HTML)
- Cover image
- Language
- Publisher
- Series information (name and number)
- Identifiers (ISBN)
- Subjects/Genres

### From MOBI Files (unencrypted)

- Book title
- Authors
- Description
- Cover image
- Language
- Publisher
- Identifiers (ISBN, ASIN)
- Subjects/Genres

### From Comic Files (CBZ/CBR)

Note: **Windows builds support CBZ only** (CBR/RAR is not supported).

- Book title (from metadata or filename)
- Series information (Series and Number)
- Authors (writers, artists, editors, etc.)
- Description (Summary)
- Publisher
- Language
- Genres
- Cover image (first image in archive)

### From KoReader Metadata

- Reading status (reading/complete)
- Highlights and annotations with chapter information
- Notes attached to highlights
- Reading progress percentage
- Rating (stars out of 5)
- Summary note (the one you can fill out at the end of the book)
- Stable page metadata (`pagemap_*`) for stable page totals and optional synthetic page scaling (nightly / post-2025.10)

### From KoReader Statistics Database (statistics.sqlite3)

- Total reading time and pages
- Weekly reading statistics
- Reading activity heatmap with customizable scaling (automatic or fixed maximum)
- Per-book reading sessions and statistics
- Reading speed calculations
- Session duration tracking
- Book completions (used by Yearly Recap)

## API Shape

KoShelf uses a model-centric API with two domains: **library** (items) and **reading** (statistics, calendar, completions). All responses are wrapped in a standard `ApiResponse<T>` envelope with `data` and `meta` fields.

| Endpoint | Description |
|----------|-------------|
| `GET /api/site` | Site metadata and capabilities |
| `GET /api/items` | Library list (supports `scope`, `sort`, `order`) |
| `GET /api/items/{id}` | Item detail (supports `include=highlights,bookmarks,statistics,completions,all`) |
| `GET /api/reading/summary` | Reading time and session aggregates |
| `GET /api/reading/metrics` | Time-series data points (daily/weekly/monthly/yearly) |
| `GET /api/reading/available-periods` | Available time periods for selectors |
| `GET /api/reading/calendar` | Monthly calendar with events and stats |
| `GET /api/reading/completions` | Book completion records with optional summary and share assets |
| `GET /api/events/stream` | SSE stream for live data invalidation |

Most endpoints accept a `scope` filter (`all`, `books`, `comics`) and reading endpoints support `from`/`to` date ranges and `tz` timezone parameters.

For full parameter documentation, response schemas, and examples, see the **[API Reference](docs/API.md)**.

In static output mode, equivalent data is exported as flat JSON files under `data/`.

## Generated Site Structure

```
site/
├── index.html              # React app shell (routes handled via HashRouter)
├── manifest.json           # PWA manifest
├── assets/
│   ├── covers/             # Optimized cover images
│   │   ├── <item-id>.webp
│   │   └── ...
│   ├── recap/              # Social media share images (generated per year)
│   │   ├── 2024_share_story.webp
│   │   ├── 2024_share_story.svg
│   │   ├── 2024_share_square.webp
│   │   ├── 2024_share_square.svg
│   │   ├── 2024_share_banner.webp
│   │   └── 2024_share_banner.svg
│   ├── css/
│   │   └── <hashed>.css    # Vite frontend bundles
│   ├── js/
│   │   └── <hashed>.js     # Vite frontend bundles
│   └── icons/              # PWA icons
│       ├── icon-192.png
│       └── icon-512.png
└── data/                   # Contract payloads used by static mode (not available when using server mode)
    ├── site.json
    ├── items/
    │   ├── index.json          # All items (list projection)
    │   ├── <item-id>.json      # Per-item detail (all includes expanded)
    │   └── ...
    └── reading/
        ├── summary.json        # Per-scope reading summaries
        ├── periods.json        # All available time periods
        ├── metrics/
        │   ├── 2024-01.json    # Daily data points per month (all scopes)
        │   └── ...
        ├── calendar/
        │   ├── 2024-01.json    # Monthly calendar data
        │   └── ...
        └── completions/
            ├── 2024.json       # Per-year completions with summary + share assets
            └── ...
```

## Credits

Design and feature inspiration taken from [KoInsight](https://github.com/GeorgeSG/KoInsight) - an excellent alternative that focuses more on statistics and also supports acting as a KOReader sync server. If you're primarily interested in reading stats rather than highlights and annotations, definitely check it out!

The calendar feature is powered by [EventCalendar](https://github.com/vkurko/calendar) - a lightweight, full-featured JavaScript event calendar library.


## Disclaimer

This was built for personal use and relies heavily on AI-generated code. While I've tested everything and use it daily, I take no responsibility for any issues you might encounter. Use at your own risk.
