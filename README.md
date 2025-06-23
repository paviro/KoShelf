# KOShelf
![Books](https://github.com/user-attachments/assets/ad096bc9-c53a-40eb-9de9-06085e854a26)
![Statistics](https://github.com/user-attachments/assets/da178885-cef9-44bc-b841-7cdcaf6a2437)
![Calendar](https://github.com/user-attachments/assets/a4ac51f1-927e-463d-b2d6-72c29fdc4323)


A Rust CLI tool that generates a beautiful static website from your KoReader library, showcasing your ebook collection with highlights, annotations, and reading progress.

## Features

- 📚 **Book Library Overview**: Displays your currently reading, completed and unread books (EPUBs only!)
- 🎨 **Modern UI**: Beautiful design powered by Tailwind CSS with clean typography and responsive layout
- 📝 **Annotations & Highlights**: Shows all your KoReader highlights and notes with elegant formatting
- 📖 **Book Details**: Individual pages for each book with metadata and organized annotations
- 📊 **Reading Statistics**: Track your reading habits with detailed statistics including reading time, pages read, activity heatmaps, and weekly breakdowns
- 📅 **Reading Calendar**: Monthly calendar view showing your reading activity with books read on each day and monthly statistics
- 📈 **Per-Book Statistics**: Detailed statistics for each book including session count, average session duration, reading speed, and last read date
- 🔍 **Search & Filter**: Search through your library by title, author, or series, with filters for reading status
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 🖥️ **Server Mode**: Built-in web server with live file watching for use with reverse proxy
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts


## Installation

### Prebuilt Binaries

The easiest way to get started is to download a prebuilt binary from the [releases page](https://github.com/paviro/koshelf/releases). Binaries are available for:

- Windows (x64)
- macOS (Apple Silicon, Intel & Universal)
- Linux (x64 and ARM64)

Download the appropriate binary for your system and make it executable:

```bash
# macOS/Linux
chmod +x koshelf
./koshelf --help
```

**Note for macOS users**: The binary is not code-signed, so macOS Gatekeeper will prevent it from running initially. You'll see a warning like "cannot be opened because the developer cannot be verified." To run the binary:

1. Run `xattr -d com.apple.quarantine koshelf` to remove the quarantine attribute, or  
2. Open it and go to System Preferences > Security & Privacy > General and click "Allow Anyway" after the first blocked attempt

### Home Assistant

Using Home Assistant? Install KOShelf as an add-on with just one click below.

[![Open your Home Assistant instance and show the dashboard of an add-on.](https://my.home-assistant.io/badges/supervisor_addon.svg)](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

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
- `--output, -o`: Output directory for the generated site (default: "site")
- `--watch, -w`: Enable file watching with static output (requires --output)
- `--title, -t`: Site title (default: "KoShelf")
- `--include-unread`: Include unread books (EPUBs without KoReader metadata) in the generated site
- `--port, -p`: Port for web server mode (default: 3000)

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
- Reading activity heatmap
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
│   ├── book-id1/           
│   │   └── index.html      # Book detail page with annotations
│   └── book-id2/
│       └── index.html
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
    └── json/               # Statistics data (when available)
        ├── week_0.json     # Weekly statistics data
        ├── week_1.json
        ├── ...
        ├── calendar_data.json # Calendar events and book data
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
