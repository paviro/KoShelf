# KoShelf

A Rust CLI tool that generates a beautiful static website from your KoReader library, showcasing your ebook collection with highlights, annotations, and reading progress.

## Features

- 📚 **Book Library Overview**: Displays your currently reading and completed books
- 🎨 **Modern UI**: Beautiful design powered by Tailwind CSS with clean typography and responsive layout
- 📝 **Annotations & Highlights**: Shows all your KoReader highlights and notes with elegant formatting
- 📖 **Book Details**: Individual pages for each book with metadata and organized annotations
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts
- ⚡ **Performance**: Fast loading with optimized images and minified CSS


## Installation

### From Source

#### Prerequisites

- Rust 1.70+ (for building)
- Node.js and npm (for Tailwind CSS compilation)

### Building the tool

```bash
git clone <repository-url>
cd koshelf

# Install Node.js dependencies for Tailwind CSS
npm install

# Build the Rust binary
cargo build --release
```

The binary will be available at `target/release/koshelf`.

**Note:** The application will automatically compile Tailwind CSS during site generation, but you can also manually build the CSS with:

```bash
# Development (with watch mode)
npm run build-css

# Production (minified)
npm run build-css-prod
```

## Usage

### Basic Usage

```bash
./target/release/koshelf --books-path /path/to/your/books --output ./my-library-site
```

### Operation Modes

KoShelf can operate in several modes:

1. **Static Site Generation**: Generate a static site once and exit (default when `--output` is specified without `--watch`)
2. **Web Server Mode**: Builds a static site in a temporary folder and serves it, automatically rebuilds on book changes (default when `--output` is not specified) 
3. **Watch Mode**: Generate a static site, rebuilding when book files change (when both `--output` and `--watch` are specified)

### Command Line Options

- `--books-path, -b`: Path to your folder containing EPUB files and KoReader metadata
- `--output, -o`: Output directory for the generated site (default: "site")
- `--watch`: Enable file watching with static output (requires --output)
- `--title, -t`: Site title (default: "KoShelf")
- `--include-unread`: Include unread books (EPUBs without KoReader metadata) in the generated site
- `--port, -p`: Port for web server mode (default: 3000)

### Example

```bash
# Generate site from Books folder
./target/release/koshelf -b ~/Books -o ~/my-reading-site -t "My Reading Journey"

# Generate site with unread books included
./target/release/koshelf -b ~/Books -o ~/my-reading-site --include-unread

# Start web server with live file watching
./target/release/koshelf -b ~/Books -p 8080

# Generate static site with file watching
./target/release/koshelf -b ~/Books -o ~/my-reading-site --watch
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

## Generated Site Structure

```
site/
├── index.html              # Main library page
├── books/                  # Individual book pages
│   ├── book-id1/           
│   │   └── index.html      # Book detail page with annotations
│   └── book-id2/
│       └── index.html
├── assets/
│   ├── covers/             # Optimized book covers
│   │   ├── book-id1.jpg
│   │   └── book-id2.jpg
│   ├── css/
│   │   └── style.css       # Tailwind CSS styling
│   └── js/
│       └── script.js       # Interactive features
```
