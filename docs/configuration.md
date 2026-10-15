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
- `--enable-writeback`: Enable metadata writeback — allows editing annotations, review notes, ratings, and reading status from the UI, with changes written back to KOReader sidecar files
- `--enable-library-management`: Enable library management — allows removing archived books from the library via the UI. Ebooks and KOReader sidecars are never modified
- `--trusted-proxies`: Comma-separated or repeated trusted reverse proxy IP/CIDR entries for forwarded client IP/proto resolution

Requires `--data-path` for persistent data storage.

### `koshelf export <output-dir>`

Generate a static site to the given directory.

**Export-specific options:**

- `--include-files`: Copy original item files into `assets/files/` (default: `false`)
- `-w, --watch`: Re-export on library changes

The output directory can also be provided via the `KOSHELF_OUTPUT` env var or `[output].path` in the TOML config.

> **Note:** `--include-files` is useful for static hosting when you want direct downloads, but it can significantly increase export size because full source files are copied. In `serve` mode, file assets are served from runtime media storage; with auth enabled, `/assets/**` (including `/assets/files/**`) requires login. Static exports have no built-in authentication, so protect hosted exports with your web server/CDN auth layer when file downloads should not be public.

### `koshelf set-password`

Set or rotate the serve-mode authentication password.

```
koshelf set-password [--data-path <PATH>] [--password <VALUE> | --random] [--overwrite]
```

For full password workflow details and examples, see [Authentication](authentication.md#set-password-command).

### `koshelf list-languages`

Print all supported UI locales and exit.

### `koshelf licenses [dependency]`

Print third-party dependency licenses. Without arguments, lists all dependencies grouped by SPDX license identifier. Pass a dependency name to view its full license text (supports exact and substring matching).

```bash
# List all dependencies and their licenses
koshelf licenses

# View the full license text for a specific dependency
koshelf licenses serde
```

### `koshelf github`

Print the repository URL and exit.

## Common Options

These flags are shared by both `serve` and `export`:

**Global (before subcommand):**

- `-c, --config`: Path to a TOML configuration file (`koshelf.toml` is auto-loaded when present)

**Library source:**

- `-i, --library-path`: Path(s) to folders containing ebooks (EPUB, FB2, MOBI) and/or comics (CBZ, CBR) with KoReader metadata. Can be specified multiple times. (optional if `--statistics-db` is provided)
- `--docsettings-path`: Path to KOReader's `docsettings` folder for users who store metadata separately (requires `--library-path`, may be combined with `--hashdocsettings-path`)
- `--hashdocsettings-path`: Path to KOReader's `hashdocsettings` folder for users who store metadata by content hash (requires `--library-path`, may be combined with `--docsettings-path`)
- `--document-metadata-folder <doc|dir|hash>`: KOReader's "Book metadata location" setting (`document_metadata_folder`), used to find and create custom metadata and covers. Default: `doc`. See [Custom Metadata and Covers](koreader-setup.md#custom-metadata-and-covers).
- `-s, --statistics-db`: Path to the `statistics.sqlite3` file for additional reading stats (optional if `--library-path` is provided). Can be specified multiple times to merge stats from several devices that don't sync their statistics database — see [Reading Statistics from Multiple Devices](koreader-setup.md#reading-statistics-from-multiple-devices). The `KOSHELF_STATISTICS_DB` environment variable accepts a single path only; use repeated flags or the config file for multiple databases.
- `--kobo-db`: Path to Kobo's `KoboReader.sqlite` database. Used to discover matched extensionless kepub files and parse them as EPUBs. Requires `--library-path`.
- `--include-unread`: Include unread items (files without KoReader metadata)

**Data:**

- `--data-path`: Persistent runtime data directory. Required for `serve`, optional for `export` (uses a temp dir when omitted).
- `--archive-enabled <true|false>`: Enable archive preservation with persistent storage (default: `true`). Also set via `KOSHELF_ARCHIVE_ENABLED` or `[library] archive_enabled`. Restart after changing this setting.
- `--archive-retention-days`: How long to keep files of removed archived books: a number of days, `0` for immediate deletion, or `forever` (default: `30`). See [Library preservation](#library-preservation).

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
- `KOSHELF_KOBO_DB`
- `KOSHELF_DOCUMENT_METADATA_FOLDER`
- `KOSHELF_OUTPUT`
- `KOSHELF_INCLUDE_FILES`
- `KOSHELF_DATA_PATH`
- `KOSHELF_ARCHIVE_RETENTION_DAYS`
- `KOSHELF_ENABLE_AUTH`
- `KOSHELF_ENABLE_WRITEBACK`
- `KOSHELF_ENABLE_LIBRARY_MANAGEMENT`
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

# Start web server with writeback enabled (edit annotations and metadata from the UI)
koshelf serve -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --data-path ~/koshelf-data --enable-writeback

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

# Include extensionless Kobo kepubs from a mounted Kobo device/library
koshelf export ~/my-reading-site -i /Volumes/KOBOeReader --kobo-db /Volumes/KOBOeReader/.kobo/KoboReader.sqlite --include-unread

# Generate site with German UI language
koshelf export ~/my-reading-site -i ~/Library --language de_DE

# Generate site with downloadable original item files
koshelf export ~/my-reading-site -i ~/Library --include-files

# Ignore stable metadata page totals and synthetic scaling
koshelf export ~/my-reading-site -i ~/Library -s ~/KOReaderSettings/statistics.sqlite3 --ignore-stable-page-metadata

# ── Password management ──────────────────────────────────────

# Set/rotate auth password explicitly
koshelf set-password --data-path ~/koshelf-data --overwrite

# Rotate to a generated random password
koshelf set-password --data-path ~/koshelf-data --overwrite --random
```

### Migrating KOReader metadata locations

KOReader supports three metadata locations: adjacent `.sdr` folders, `docsettings`, and `hashdocsettings`. It changes locations gradually as books are saved. KoShelf always considers adjacent sidecars, plus whichever central directories you explicitly configure. To support migration among all three locations:

```sh
koshelf serve --library-path ~/Books \
  --docsettings-path ~/KOReaderSettings/docsettings \
  --hashdocsettings-path ~/KOReaderSettings/hashdocsettings \
  --data-path ~/koshelf-data
```

The same settings can be supplied through `KOSHELF_DOCSETTINGS_PATH` and `KOSHELF_HASHDOCSETTINGS_PATH`, or together in TOML:

```toml
[library]
paths = ["/home/me/Books"]
docsettings_path = "/home/me/KOReaderSettings/docsettings"
hashdocsettings_path = "/home/me/KOReaderSettings/hashdocsettings"
```

Each setting retains its normal CLI/environment/TOML precedence. KoShelf always searches adjacent `.sdr` folders and only searches central directories that you explicitly configure; it never infers an unconfigured central directory. Without either central setting, discovery remains adjacent-only. Restart with the changed configuration to switch configured directories.

Modern sidecar selection is checked against [KOReader v2026.07.1](https://github.com/koreader/koreader/blob/v2026.07.1/frontend/docsettings.lua). KoShelf intentionally preserves invalid files and old metadata locations rather than deleting or migrating them as KOReader can on open/save. Legacy sidecars, history files, and `.kpdfview` files are outside this compatibility scope.

For each ebook, KoShelf considers valid primary files and their `.old` backups using KOReader's ordering. Candidates sort newest first using whole-second timestamps, matching KOReader’s LuaFileSystem; ties prefer adjacent primary/backup, then docsettings primary/backup, then hashdocsettings primary/backup. If a backup is newer than its primary, the primary receives the backup's ranking timestamp so it is tried first. Invalid or empty candidates are skipped without deleting them. Hash lookup uses the ebook's calculated checksum and expected metadata filename. Ambiguous associations and conflicting identities block editing rather than choosing arbitrarily. Legacy history files and `.kpdfview` files are not imported.

When the selected file disappears or becomes invalid, KoShelf accepts the next valid candidate, including an older primary or backup. Valid metadata without annotations clears previous annotations. When no valid source remains, KoShelf keeps showing the last saved reading data with editing disabled (see [Library preservation](#library-preservation)).

Reading a backup does not restore or modify files. The editor identifies backup-derived data; an explicit edit restores the primary in the same directory using the backup's contents plus the requested change. The backup remains untouched. A changed source, backup, or destination causes a conflict and requires reviewing refreshed data before another edit. Open editors preserve drafts and prevent saving against a revision that changed in the background. Backup file events trigger reevaluation just like primary file events.

Unavailable central directories do not stop library watching. KoShelf watches the nearest available ancestor and retries central watch registration every five seconds; restored directories trigger reconciliation.


## Library preservation

Archive mode is enabled by default when a persistent data path is configured. To disable it, set `archive_enabled = false` under `[library]`, use `KOSHELF_ARCHIVE_ENABLED=false`, or pass `--archive-enabled false`. CLI values override environment variables, which override TOML values.

While disabled, missing books are removed from the visible library, archive filters and removal controls are hidden, and no saved bundles are loaded, updated, recovered, or expired. Existing saved files remain untouched; startup logs an informational message with their location when any exist. Library management is unavailable even when explicitly enabled. Configured library roots must exist at startup.

Restart to apply the setting. Re-enabling restores saved archives and resumes normal reconciliation and retention cleanup, including cleanup of retired files that became due while disabled. Persistent storage still uses an exclusive writer lock in either mode. Exports accept the same setting; without persistent storage, archiving is always inactive.

With a persistent `--data-path` (always the case for `serve`), KoShelf saves a copy of each book's KOReader metadata and cover under `<data-path>/library/`. If an ebook later disappears (deleted, moved, or on an unplugged drive), it stays in your library as **Archived** with its status, annotations, and statistics intact. Reading, downloading, and editing are unavailable until the ebook returns, at which point it becomes available again automatically.

Missing or inaccessible library roots do not prevent startup when persistent archives are enabled. Warnings identify the affected path and filesystem error, with guidance to reconnect the drive or check permissions. In watch mode, KoShelf retries automatically, suppresses repeated identical watch warnings, and logs when access is restored. A path that is confirmed to be a file instead of a directory remains a configuration error.

- The saved copy mirrors the current KOReader metadata; it is not a version history.
- Archived books never expire on their own. You can remove them from the book's page (requires `--enable-library-management`). Original ebooks and sidecars are never touched.
- A removed book reappears if its ebook shows up again.
- Exports without `--data-path` are ephemeral and preserve nothing.

### Removal retention

`--archive-retention-days` (`KOSHELF_ARCHIVE_RETENTION_DAYS`, `[library] archive_retention_days`) controls how long saved files are kept after you remove an archived book:

| Value | After removal |
|---|---|
| Positive number (default `30`) | Files are kept for that many days, then deleted |
| `0` | Files are deleted immediately |
| `forever` | Files are kept indefinitely |

Removed books disappear from the library immediately regardless of this setting.

### Recovering metadata manually

Each book is stored in its own directory, `<data-path>/library/{snapshots,archived,retired}/<id>/`, containing the original KOReader metadata file (e.g. `metadata.epub.lua`) and `cover.webp`. To restore, copy the metadata file back into the book's `.sdr` folder (or its docsettings/hashdocsettings location). Don't modify these directories otherwise, and only run one KoShelf process per data path.
