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
- 🎯 **Accessibility**: Semantic HTML and proper contrast ratios

## Prerequisites

- Rust 1.70+ (for building)
- Node.js and npm (for Tailwind CSS compilation)
- KoReader with EPUB files and `.sdr` metadata directories

## Installation

### From Source

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

### Command Line Options

- `--books-path, -b`: Path to your folder containing EPUB files and KoReader metadata
- `--output, -o`: Output directory for the generated site (default: "site")
- `--title, -t`: Site title (default: "My KoShelf Library")

### Example

```bash
# Generate site from Books folder
./target/release/koshelf -b ~/Books -o ~/my-reading-site -t "My Reading Journey"

# Serve the generated site locally (optional)
cd ~/my-reading-site
python3 -m http.server 8000
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

## Supported Data

### From EPUB Files
- Book title
- Authors
- Description
- Cover image

### From KoReader Metadata
- Reading status (reading/complete)
- Highlights and annotations
- Reading progress
- Page numbers
- Chapter information
- Timestamps

## Generated Site Structure

```
site/
├── index.html              # Main library page
├── books/                  # Individual book pages
│   ├── book-1.html
│   └── book-2.html
├── covers/                 # Optimized book covers
│   ├── book-1.jpg
│   └── book-2.jpg
├── css/
│   └── style.css           # Modern styling
└── js/
    └── script.js           # Interactive features
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Development Mode

```bash
cargo run -- --books-path ./test-books --output ./test-site
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [KoReader](https://koreader.rocks/) - The amazing open-source e-reader software
- Built with Rust for performance and reliability 