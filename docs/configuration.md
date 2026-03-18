# Configuration

## Quick Start

```bash
# Start a web server (live file watching, requires --data-path)
koshelf serve -i ~/Library --data-path ~/koshelf-data

# Generate a static site
koshelf export ~/my-reading-site -i ~/Library
```

## Subcommands

KoShelf uses explicit subcommands — run `koshelf <command> --help` for full flag details.

### `koshelf serve`

Start the web server. Serves the embedded React app at `/` with API endpoints under `/api/**`, and automatically refreshes data on library changes.

**Serve-specific options:**

- `-p, --port`: Port for web server (default: 3000)
- `--enable-auth`: Enable password authentication
- `--trusted-proxies`: Comma-separated or repeated trusted reverse proxy IP/CIDR entries for forwarded client IP/proto resolution

Requires `--data-path` for persistent data storage.

### `koshelf export <output-dir>`

Generate a static site to the given directory.

**Export-specific options:**

- `-w, --watch`: Re-export on library changes

The output directory can also be provided via the `KOSHELF_OUTPUT` env var or `[output].path` in the TOML config.

### `koshelf set-password`

Set or rotate the serve-mode authentication password.

```
koshelf set-password [--data-path <PATH>] [--password <VALUE> | --random] [--overwrite]
```

For full password workflow details and examples, see [Authentication](authentication.md#set-password-command).

### `koshelf list-languages`

Print all supported UI locales and exit.

### `koshelf github`

Print the repository URL and exit.

## Common Options

These flags are shared by both `serve` and `export`:

**Global (before subcommand):**

- `-c, --config`: Path to a TOML configuration file (`koshelf.toml` is auto-loaded when present)

**Library source:**

- `-i, --library-path`: Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata. Can be specified multiple times. (optional if `--statistics-db` is provided)
- `--docsettings-path`: Path to KOReader's `docsettings` folder for users who store metadata separately (requires `--library-path`, mutually exclusive with `--hashdocsettings-path`)
- `--hashdocsettings-path`: Path to KOReader's `hashdocsettings` folder for users who store metadata by content hash (requires `--library-path`, mutually exclusive with `--docsettings-path`)
- `-s, --statistics-db`: Path to the `statistics.sqlite3` file for additional reading stats (optional if `--library-path` is provided)
- `--include-unread`: Include unread items (files without KoReader metadata)

**Data:**

- `--data-path`: Persistent runtime data directory. Required for `serve`, optional for `export` (uses a temp dir when omitted).

**Site display:**

- `-t, --title`: Site title (default: "KoShelf")
- `-l, --language`: Default server language for UI translations. Frontend language/region settings can override this per browser. Use full locale code (e.g., `en_US`, `de_DE`, `pt_BR`) for correct date formatting. Default: `en_US`
- `--timezone`: Timezone to interpret timestamps (IANA name, e.g., `Australia/Sydney`); defaults to system local

**Statistics tuning:**

- `--heatmap-scale-max`: Maximum value for heatmap color intensity scaling (e.g., "auto", "1h", "1h30m", "45min"). Values above this will still be shown but use the highest color intensity. Default is `2h` (pass `auto` for automatic scaling)
- `--day-start-time`: Logical day start time as `HH:MM` (default: `00:00`)
- `--min-pages-per-day`: Minimum pages read per book per day to be counted in statistics (optional)
- `--min-time-per-day`: Minimum reading time per book per day to be counted in statistics (e.g., "30s", "15m", "1h", `off`). Default is `30s`.
    > **Note:** If both `--min-pages-per-day` and `--min-time-per-day` are provided, a book's data for a day is counted if **either** condition is met for that book on that day. These filters apply **per book per day**, meaning each book must individually meet the threshold for each day to be included in statistics. Since `--min-time-per-day` defaults to `30s`, it is active unless explicitly overridden. Use `--min-time-per-day off` to disable this filter.
- `--include-all-stats`: By default, statistics are filtered to only include books present in your `--library-path` directories. This prevents deleted books or external files (like Wallabag articles) from skewing your recap and statistics. Use this flag to include statistics for all books in the database, regardless of whether they exist in your library.
- `--ignore-stable-page-metadata`: Ignore KOReader stable page metadata for page totals and page-based stats scaling. By default, stable metadata is used when available. See [Stable Page Metadata](stable-page-metadata.md) for details.

## Configuration Sources & Precedence

Settings are merged in this order (highest priority first):

1. CLI arguments
2. Environment variables
3. Config file (`--config` or default `koshelf.toml`)
4. Built-in defaults

### Environment Variables

Common environment variables:

- `KOSHELF_LIBRARY_PATH`
- `KOSHELF_STATISTICS_DB`
- `KOSHELF_OUTPUT`
- `KOSHELF_DATA_PATH`
- `KOSHELF_ENABLE_AUTH`
- `KOSHELF_TRUSTED_PROXIES`
- `KOSHELF_TITLE`
- `KOSHELF_LANGUAGE`

Run `koshelf serve --help` or `koshelf export --help` to see the full env mapping for every option.

For `set-password`, data path resolution is: `--data-path` > `KOSHELF_DATA_PATH` > config file.

### TOML Config File

KoShelf auto-loads `koshelf.toml` from the current directory when present. See [`koshelf.example.toml`](../koshelf.example.toml) for the full template.

## Examples

```bash
# ── Serve mode ────────────────────────────────────────────────

# Start web server with live file watching and statistics
koshelf serve -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --data-path ~/koshelf-data -p 8080

# Start web server with authentication enabled (password generated on first run)
koshelf serve -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --data-path ~/koshelf-data --enable-auth

# ── Export mode ───────────────────────────────────────────────

# Generate site from a library folder
koshelf export ~/my-reading-site -i ~/Library -t "My Reading Journey"

# Generate site from multiple folders (e.g., books + comics)
koshelf export ~/my-reading-site -i ~/Books -i ~/Comics

# Generate site with statistics and unread items included
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --include-unread

# Generate static site with file watching and statistics
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --watch

# Generate site with custom heatmap color scaling (3 hours = highest intensity)
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --heatmap-scale-max 3h

# Generate site with explicit timezone and non-midnight day start (good for night owls)
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --timezone Australia/Sydney --day-start-time 03:00

# Using hashdocsettings (metadata stored by content hash)
koshelf export ~/my-reading-site -i ~/Books --hashdocsettings-path ~/KOReaderSettings/hashdocsettings

# Using docsettings (metadata stored in central folder by path)
koshelf export ~/my-reading-site -i ~/Books --docsettings-path ~/KOReaderSettings/docsettings

# Generate site with German UI language
koshelf export ~/my-reading-site -i ~/Library --language de_DE

# Ignore stable metadata page totals and synthetic scaling
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --ignore-stable-page-metadata

# ── Password management ──────────────────────────────────────

# Set/rotate auth password explicitly
koshelf set-password --data-path ~/koshelf-data --overwrite

# Rotate to a generated random password
koshelf set-password --data-path ~/koshelf-data --overwrite --random
```
