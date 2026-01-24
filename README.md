<div align="center">

# KoShelf

<p>
  <a href="https://github.com/zanivann/koshelf/stargazers">
    <img src="https://img.shields.io/github/stars/zanivann/koshelf?style=social" alt="Stars" />
  </a>
  <a href="https://github.com/zanivann/koshelf/releases/latest">
    <img src="https://img.shields.io/github/v/release/zanivann/koshelf?label=release" alt="Latest Release" />
  </a>
  <a href="https://github.com/zanivann/koshelf/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/zanivann/koshelf" alt="License" />
  </a>
</p>

![Statistics dashboard](https://github.com/user-attachments/assets/94a094d2-298b-412c-80b3-b3b2e2cfc6de)

######  A Rust CLI tool that generates a beautiful static website from your KoReader library, showcasing your ebook collection with highlights, annotations, and reading progress.

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
  - [Example](#example)
- [KoReader Setup](#koreader-setup)
  - [Metadata Storage Options](#metadata-storage-options)
  - [Typical Deployment Setup](#typical-deployment-setup)
- [Supported Data](#supported-data)
  - [From EPUB Files](#from-epub-files)
  - [From KoReader Metadata](#from-koreader-metadata)
  - [From KoReader Statistics Database](#from-koreader-statistics-database-statisticssqlite3)
- [Generated Site Structure](#generated-site-structure)
- [Credits](#credits)
- [Disclaimer](#disclaimer)

## Features

- 📚 **Library Overview (Books + Comics)**: Displays your currently reading, completed, and unread items (ebooks + comics)
- 🎨 **Modern UI**: Beautiful design powered by Tailwind CSS with clean typography and responsive layout
- 📝 **Annotations, Highlights & Ratings**: All your KoReader highlights, notes, star ratings, and review notes (summary note) are shown together on each book's details page with elegant formatting
- 📊 **Reading Statistics**: Track your reading habits with detailed statistics including reading time, pages read, customizable activity heatmaps, and weekly breakdowns
- 📅 **Reading Calendar**: Monthly calendar view showing your reading activity with books read on each day and monthly statistics
- 🎉 **Yearly Recap**: Celebrate your reading year with a timeline of completions, monthly summaries (finished books, hours read), and rich per‑book details
- 📈 **Per-Book Statistics**: Detailed statistics for each book including session count, average session duration, reading speed, and last read date
- 🔍 **Search & Filter**: Search through your library by title, author, or series, with filters for reading status
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 🖥️ **Server Mode**: Built-in web server with live file watching for use with reverse proxy
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts
- 🔌 **API**: New API to external access of data and statstcs of books

## Screenshots

![Library overview](https://github.com/user-attachments/assets/ad096bc9-c53a-40eb-9de9-06085e854a26)
![Book details](https://github.com/user-attachments/assets/44113be0-aa19-4018-b864-135ddb067a9d)
![Reading calendar](https://github.com/user-attachments/assets/a4ac51f1-927e-463d-b2d6-72c29fdc4323)
![Recap](https://github.com/user-attachments/assets/9558eea9-dee1-4b0a-adac-1bc0157f0181)




## Installation

### Docker Compose Deployment
Deploy KoShelf easily using the community-maintained Docker image.
#### Quick Start
1. Create a docker-compose.yml file:

```services:
  koshelf:
    image: ghcr.io/devtigro/koshelf:latest
    ports:
     - "3000:3000"
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

4. Access KoShelf at http://localhost:3009

Docker Image Repository: [koshelf-docker](https://github.com/DevTigro/koshelf-docker)

### CasaOS
1. The file must be imported in app personalized instaltion of CasaOS:
```name: koshelf-zanivann
services:
  koshelf:
    container_name: koshelf-zanivann
    image: ghcr.io/zanivann/koshelf:latest
    cpu_shares: 90
    command:
      - --library-path
      - /books
      - --statistics-db
      - /settings/statistics.sqlite3
      - --port
      - "3009"
      - --timezone
      - America/Sao_Paulo
      - --include-unread
      - --language
      - pt_BR
    ports:
      - target: 3009
        published: "3009"
        protocol: tcp
    restart: unless-stopped
    volumes:
      - type: bind
        source: /DATA/AppData/koshelf-books
        target: /books
      - type: bind
        source: /DATA/AppData/koshelf
        target: /settings
    x-casaos:
      icon: https://b.thumbs.redditmedia.com/Flac-ySmslzY0SE583PNA42rFbcYxLt7hqgCeUrC11s.png
      title: KoShelf Personalizado
    network_mode: bridge
x-casaos:
  author: zanivann
  category: self
  icon: https://b.thumbs.redditmedia.com/Flac-ySmslzY0SE583PNA42rFbcYxLt7hqgCeUrC11s.png
  index: /
  port_map: "3009"
  scheme: http
  title:
    custom: KoShelf Personalizado
```

### Prebuilt Binaries

The easiest way to get started is to download a prebuilt binary from the [releases page](https://github.com/zanivann/koshelf/releases). Binaries are available for:

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
- Node.js and npm (for Tailwind CSS compilation)

#### Building the tool

```bash
git clone https://github.com/zanivann/KoShelf
cd KoShelf

# Build the Rust binary
cargo build --release
```

The binary will be available at `target/release/koshelf`.

**Note:** Tailwind CSS will be compiled during build and added to the binary.

## Usage

### Basic Usage

```bash
./koshelf --library-path /path/to/your/library --output ./my-library-site
```

### Operation Modes

KoShelf can operate in several modes:

1. **Static Site Generation**: Generate a static site once and exit (default when `--output` is specified without `--watch`)
2. **Web Server Mode**: Builds a static site in a temporary folder and serves it, automatically rebuilds on book changes (default when `--output` is not specified) 
3. **Watch Mode**: Generate a static site, rebuilding when book files change (when both `--output` and `--watch` are specified)

### Command Line Options

- `-i, --library-path`: Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata. Can be specified multiple times. (optional if `--statistics-db` is provided)
- `--docsettings-path`: Path to KOReader's `docsettings` folder for users who store metadata separately (requires `--books-path`, mutually exclusive with `--hashdocsettings-path`)
- `--hashdocsettings-path`: Path to KOReader's `hashdocsettings` folder for users who store metadata by content hash (requires `--books-path`, mutually exclusive with `--docsettings-path`)
- `-s, --statistics-db`: Path to the `statistics.sqlite3` file for additional reading stats (optional if `--books-path` is provided)
- `-o, --output`: Output directory for the generated site
- `-p, --port`: Port for web server mode (default: 3009)
- `-w, --watch`: Enable file watching with static output (requires `--output`)
- `-t, --title`: Site title (default: "KoShelf")
- `--include-unread`: Include unread items (files without KoReader metadata)
- `--heatmap-scale-max`: Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is "auto" for automatic scaling
- `--timezone`: Timezone to interpret timestamps (IANA name, e.g., `Australia/Sydney`); defaults to system local
- `--day-start-time`: Logical day start time as `HH:MM` (default: `00:00`)
- `--min-pages-per-day`: Minimum pages read per book per day to be counted in statistics (optional)
- `--min-time-per-day`: Minimum reading time per book per day to be counted in statistics (e.g., "15m", "1h") (optional)
  > **Note:** If both `--min-pages-per-day` and `--min-time-per-day` are provided, a book's data for a day is counted if **either** condition is met for that book on that day. These filters apply **per book per day**, meaning each book must individually meet the threshold for each day to be included in statistics.
- `--include-all-stats`: By default, statistics are filtered to only include books present in your `--books-path` directory. This prevents deleted books or external files (like Wallabag articles) from skewing your recap and statistics. Use this flag to include statistics for all books in the database, regardless of whether they exist in your library.
- `-l, --language`: Language for UI translations. Use full locale code (e.g., `en_US`, `de_DE`, `pt_BR`) for correct date formatting. Default: `en_US`
- `--list-languages`: List all supported languages and exit
- `--github`: Print GitHub repository URL

### Example

```bash
# Generate site from a library folder
./koshelf -i ~/Library -o ~/my-reading-site -t "My Reading Journey"

# Generate site from multiple folders (e.g., books + comics)
./koshelf -i ~/Books -i ~/Comics -o ~/my-reading-site

# Generate site with statistics and unread items included
./koshelf -i ~/Library -o ~/my-reading-site --statistics-db ~/KOReaderSettings/statistics.sqlite3 --include-unread

# Start web server with live file watching and statistics
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -p 8080

# Generate static site with file watching and statistics
./koshelf --library-path ~/Library -o ~/my-reading-site --statistics-db ~/KOReaderSettings/statistics.sqlite3 --watch

# Generate site with custom heatmap color scaling (2 hours = highest intensity)
./koshelf -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --heatmap-scale-max 2h

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
```

🔌 Developer API

This fork includes a dedicated REST API endpoint to fetch your library statistics programmatically for use in external dashboards or personal websites.

Endpoint: GET /api/stats

Example Request:
curl http://localhost:3009/api/stats

Response JSON:

```{
  "total": 16,
  "read": 8,
  "reading": 4,
  "paused": 2,
  "unread": 2
}```


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

**Note:**  Unlike KOReader, KOShelf matches books by filename only, since the folder structure reflects the device path (which may differ from your local path). If you have multiple books with the same filename, KOShelf will show an error - use `hashdocsettings` or `book folder` instead.

**Usage:**
```bash
./koshelf --library-path ~/Books --docsettings-path ~/KOReaderSettings/docsettings
```

### Typical Deployment Setup

Although there are many ways to use this tool here is how I use it:

1. **Syncthing Sync**: I use [Syncthing](https://syncthing.net/) to sync both my books folder and KoReader settings folder from my e-reader to my server
2. **Books and Statistics**: I point to the synced books folder with `--books-path` and to `statistics.sqlite3` in the synced KoReader settings folder with `--statistics-db`
3. **Web Server Mode**: I then run KoShelf in web server mode (without `--output`) - it will automatically rebuild when files change
4. **Nginx Reverse Proxy**: I use an nginx reverse proxy for HTTPS and to restrict access

My actual setup:
```bash
# My server command - runs continuously with file watching and statistics
./koshelf --library-path ~/syncthing/Books \
         --statistics-db ~/syncthing/KOReaderSettings/statistics.sqlite3 \
         --port 3009
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

### From KoReader Statistics Database (statistics.sqlite3)
- Total reading time and pages
- Weekly reading statistics
- Reading activity heatmap with customizable scaling (automatic or fixed maximum)
- Per-book reading sessions and statistics
- Reading speed calculations
- Session duration tracking
- Book completions (used by Yearly Recap)

## Generated Site Structure

```
site/
├── index.html              # Main library page (books list if any books exist; otherwise comics list)
├── manifest.json           # PWA Manifest
├── service-worker.js       # PWA Service Worker
├── cache-manifest.json     # PWA Smart Cache Manifest
├── version.txt             # Version timestamp for lightweight polling
├── recap/                  # Yearly Recap pages
│   ├── index.html          # Empty state / Recap landing
│   ├── 2024/
│   │   ├── index.html      # Yearly recap page
│   │   ├── books/
│   │   │   └── index.html  # Yearly recap page (books only; only generated when both books+comics exist)
│   │   └── comics/
│   │       └── index.html  # Yearly recap page (comics only; only generated when both books+comics exist)
│   └── ...
├── statistics/
│   ├── index.html          # Reading statistics dashboard
│   ├── books/
│   │   └── index.html      # Reading statistics dashboard (books only; only generated when both books+comics exist)
│   └── comics/
│       └── index.html      # Reading statistics dashboard (comics only; only generated when both books+comics exist)
├── calendar/
│   └── index.html          # Reading calendar view
├── books/                  # Individual book pages
│   ├── list.json           # Manifest of all books (convenience only; not used by frontend)
│   ├── book-id1/           
│   │   ├── index.html      # Book detail page with annotations
│   │   ├── details.md      # Markdown export (human-readable)
│   │   └── details.json    # JSON export (machine-readable)
│   └── ...
├── comics/                 # Comics list + individual comic pages
│   ├── index.html          # Comics list page (only when books also exist; otherwise list is at /index.html)
│   ├── list.json           # Manifest of all comics (convenience only; not used by frontend)
│   ├── comic-id1/
│   │   ├── index.html      # Comic detail page with annotations
│   │   ├── details.md      # Markdown export (human-readable)
│   │   └── details.json    # JSON export (machine-readable)
│   └── ...
└── assets/
    ├── covers/             # Optimized cover images
    │   ├── book-id1.webp
    │   └── ...
    ├── recap/              # Social media share images (generated per year)
    │   ├── 2024_share_story.webp
    │   ├── 2024_share_story.svg
    │   ├── 2024_share_square.webp
    │   ├── 2024_share_square.svg
    │   ├── 2024_share_banner.webp
    │   └── 2024_share_banner.svg
    ├── css/
    │   ├── style.css       # Compiled Tailwind CSS
    │   └── event-calendar.min.css # Event calendar library styles
    ├── js/
    │   ├── library_list.js # Search and filtering functionality
    │   ├── item_detail.js  # Item detail page functionality
    │   ├── lazy-loading.js # Image lazy loading
    │   ├── section-toggle.js # Section collapsing/expanding
    │   ├── statistics.js   # Statistics page functionality
    │   ├── heatmap.js      # Activity heatmap visualization
    │   ├── calendar.js     # Calendar initialization
    │   ├── event-calendar.min.js # Event calendar library
    │   ├── recap.js        # Recap page interactions
    │   ├── storage-manager.js # Centralized local storage management
    │   └── pwa.js          # PWA installation and update logic
    ├── icons/              # PWA icons
    │   ├── icon-192.png
    │   └── icon-512.png
    └── json/               # Data files used by the frontend (for dynamic loading)
        ├── locales.json        # UI translations for the selected language
        ├── calendar/           # Calendar data split by month
        │   ├── available_months.json 
        │   ├── 2024-01.json   
        │   └── ...            
        └── statistics/         # Statistics data
            ├── all/            # Always generated when stats are enabled
            │   ├── week_0.json
            │   ├── ...
            │   └── daily_activity_2024.json
            ├── books/          # Only generated when both books+comics exist
            │   ├── week_0.json
            │   ├── ...
            │   └── daily_activity_2024.json
            └── comics/         # Only generated when both books+comics exist
                ├── week_0.json
                ├── ...
                └── daily_activity_2024.json
```

## Credits

This project is a fork of the original [paviro/KoShelf](https://github.com/paviro/KoShelf). Special thanks to:

[KoInsight](https://github.com/GeorgeSG/KoInsight) - for design inspiration.

[EventCalendar](https://github.com/vkurko/calendar) - for the calendar engine.

[Tailwind CSS](https://tailwindcss.com/) for the UI framework.

## Disclaimer

This is a weekend project and was built for personal use - it relies heavily on AI-generated code. While I've tested everything and use it daily, I take no responsibility for any issues you might encounter. Use at your own risk.
