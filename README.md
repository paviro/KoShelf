<div align="center">

# KOShelf

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
  - [Typical Deployment Setup](#typical-deployment-setup)
- [Supported Data](#supported-data)
  - [From EPUB Files](#from-epub-files)
  - [From KoReader Metadata](#from-koreader-metadata)
  - [From KoReader Statistics Database](#from-koreader-statistics-database-statisticssqlite3)
- [Generated Site Structure](#generated-site-structure)
- [Credits](#credits)
- [Disclaimer](#disclaimer)

## Features

- 📚 **Book Library Overview**: Displays your currently reading, completed and unread books (EPUBs only!)
- 🎨 **Modern UI**: Beautiful design powered by Tailwind CSS with clean typography and responsive layout
- 📝 **Annotations, Highlights & Ratings**: All your KoReader highlights, notes, star ratings, and review notes (summary note) are shown together on each book's details page with elegant formatting
- 📊 **Reading Statistics**: Track your reading habits with detailed statistics including reading time, pages read, customizable activity heatmaps, and weekly breakdowns
- 📅 **Reading Calendar**: Monthly calendar view showing your reading activity with books read on each day and monthly statistics
- 📈 **Per-Book Statistics**: Detailed statistics for each book including session count, average session duration, reading speed, and last read date
- 🔍 **Search & Filter**: Search through your library by title, author, or series, with filters for reading status
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 🖥️ **Server Mode**: Built-in web server with live file watching for use with reverse proxy
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts

## Screenshots

![Library overview](https://github.com/user-attachments/assets/ad096bc9-c53a-40eb-9de9-06085e854a26)
![Book details](https://github.com/user-attachments/assets/44113be0-aa19-4018-b864-135ddb067a9d)
![Reading calendar](https://github.com/user-attachments/assets/a4ac51f1-927e-463d-b2d6-72c29fdc4323)

## Installation

### Home Assistant

Using Home Assistant? Install KOShelf as an add-on with just one click below.

[![Open your Home Assistant instance and show the dashboard of an add-on.](https://my.home-assistant.io/badges/supervisor_addon.svg)](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

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
./koshelf --books-path /path/to/your/books --output ./my-library-site
```

**Pro tip:** On most terminals, you can drag and drop the downloaded binary file directly into the terminal window. This will automatically insert the full file path, allowing you to immediately add your arguments and run the command.

If you plan to use KoShelf frequently and use Linux or macOS, you can move the binary to `/usr/local/bin/` to make it available system-wide. This allows you to run `koshelf` from anywhere without specifying the full path:

```bash
# Move the binary to system PATH (requires sudo)
sudo mv koshelf /usr/local/bin/

# Now you can run it from anywhere
koshelf --books-path ~/Books --output ~/my-library-site
```

### From Source

If you prefer to build from source or need a custom build:

#### Prerequisites

- Rust 1.70+ (for building)
- Node.js and npm (for Tailwind CSS compilation)

#### Building the tool

```bash
git clone https://github.com/paviro/KOShelf
cd koshelf

# Build the Rust binary
cargo build --release
```

The binary will be available at `target/release/koshelf`.

**Note:** Tailwind CSS will be compiled during build and added to the binary.

## Usage

### Basic Usage

```bash
./koshelf --books-path /path/to/your/books --output ./my-library-site
```

### Operation Modes

KoShelf can operate in several modes:

1. **Static Site Generation**: Generate a static site once and exit (default when `--output` is specified without `--watch`)
2. **Web Server Mode**: Builds a static site in a temporary folder and serves it, automatically rebuilds on book changes (default when `--output` is not specified) 
3. **Watch Mode**: Generate a static site, rebuilding when book files change (when both `--output` and `--watch` are specified)

### Command Line Options

- `--books-path, -b`: Path to your folder containing EPUB files and KoReader metadata (optional if --statistics-db is provided)
- `--statistics-db, -s`: Path to the statistics.sqlite3 file for additional reading stats (optional if --books-path is provided)
- `--output, -o`: Output directory for the generated site
- `--watch, -w`: Enable file watching with static output (requires --output)
- `--title, -t`: Site title (default: "KoShelf")
- `--include-unread`: Include unread books (EPUBs without KoReader metadata)
- `--port, -p`: Port for web server mode (default: 3000)
- `--heatmap-scale-max`: Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is "auto" for automatic scaling

### Example

```bash
# Generate site from Books folder
./koshelf -b ~/Books -o ~/my-reading-site -t "My Reading Journey"

# Generate site with statistics and unread books included
./koshelf -b ~/Books -o ~/my-reading-site --s ~/KOReaderSettings/statistics.sqlite3 --include-unread

# Start web server with live file watching and statistics
./koshelf -b ~/Books --s ~/KOReaderSettings/statistics.sqlite3 -p 8080

# Generate static site with file watching and statistics
./koshelf -books-path ~/Books -o ~/my-reading-site --statistics-db ~/KOReaderSettings/statistics.sqlite3 --watch

# Generate site with custom heatmap color scaling (2 hours = highest intensity)
./koshelf -b ~/Books -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --heatmap-scale-max 2h

# Generate site with custom heatmap color scaling (1.5 hours = highest intensity)
./koshelf -b ~/Books -s ~/KOReaderSettings/statistics.sqlite3 -o ~/my-reading-site --heatmap-scale-max 1h30m
```

## KoReader Setup

This tool expects your books to be organized like this:

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

The `.sdr` directories are automatically created by KoReader when you read books and make highlights/annotations.
Although KOReader supports more than just EPUBs, this tool does not, and probably never will, as I don't use them and this is a weekend project that probably won't be maintained much.

### Typical Deployment Setup

Although there are many ways to use this tool here is how I use it:

1. **Syncthing Sync**: I use [Syncthing](https://syncthing.net/) to sync both my books folder and KoReader settings folder from my e-reader to my server
2. **Books and Statistics**: I point to the synced books folder with `--books-path` and to `statistics.sqlite3` in the synced KoReader settings folder with `--statistics-db`
3. **Web Server Mode**: I then run KoShelf in web server mode (without `--output`) - it will automatically rebuild when files change
4. **Nginx Reverse Proxy**: I use an nginx reverse proxy for HTTPS and to restrict access

My actual setup:
```bash
# My server command - runs continuously with file watching and statistics
./koshelf --books-path ~/syncthing/Books \
         --statistics-db ~/syncthing/KOReaderSettings/statistics.sqlite3 \
         --port 3000
```

This way, every time Syncthing pulls updates from my e-reader, the website automatically updates with my latest reading progress, new highlights, and updated statistics.

## Supported Data

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

## Generated Site Structure

```
site/
├── index.html              # Main library page
├── statistics/
│   └── index.html          # Reading statistics dashboard
├── calendar/
│   └── index.html          # Reading calendar view
├── books/                  # Individual book pages
│   ├── list.json           # Manifest of all books (convenience only; not used by frontend)
│   ├── book-id1/           
│   │   ├── index.html      # Book detail page with annotations
│   │   ├── details.md      # Markdown export (human-readable)
│   │   └── details.json    # JSON export (machine-readable)
│   └── book-id2/
│       ├── index.html
│       ├── details.md
│       └── details.json
└── assets/
    ├── covers/             # Optimized book covers
    │   ├── book-id1.webp
    │   └── book-id2.webp
    ├── css/
    │   ├── style.css       # Compiled Tailwind CSS
    │   └── event-calendar.min.css # Event calendar library styles
    ├── js/
    │   ├── book_list.js    # Search and filtering functionality
    │   ├── lazy-loading.js # Image lazy loading
    │   ├── statistics.js   # Statistics page functionality
    │   ├── calendar.js     # Calendar functionality
    │   ├── heatmap.js      # Activity heatmap visualization
    │   └── event-calendar.min.js # Event calendar library
    └── json/               # Data files used by the frontend (when available)
        ├── calendar/           # Calendar data split by month
        │   ├── available_months.json # List of months with calendar data
        │   ├── 2024-01.json   # January 2024 events and book data
        │   ├── 2024-02.json   # February 2024 events and book data
        │   └── ...            # Additional monthly files
        └── statistics/         # Statistics data
            ├── week_0.json     # Weekly statistics data
            ├── week_1.json
            ├── ...
            ├── daily_activity_2023.json # Daily activity data for heatmap
            ├── daily_activity_2024.json
            └── ...
```

## Credits

Design and feature inspiration taken from [KoInsight](https://github.com/GeorgeSG/KoInsight) - an excellent alternative that focuses more on statistics and also supports acting as a KOReader sync server. If you're primarily interested in reading stats rather than highlights and annotations, definitely check it out!

The calendar feature is powered by [EventCalendar](https://github.com/vkurko/calendar) - a lightweight, full-featured JavaScript event calendar library.

Styled with [Tailwind CSS](https://tailwindcss.com/) for modern, responsive design.

## Disclaimer

This is a weekend project and was built for personal use - it relies heavily on AI-generated code. While I've tested everything and use it daily, I take no responsibility for any issues you might encounter. Use at your own risk.
