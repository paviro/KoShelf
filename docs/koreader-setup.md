# KoReader Setup

## Metadata Storage Options

KOReader offers three ways to store book metadata (reading progress, highlights, annotations). KoShelf supports all three.

### 1. Default: Metadata Next to Books (Recommended)

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

This is the simplest setup — just point `--library-path` to your books folder.

### 2. Hashdocsettings

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
koshelf serve --library-path ~/Books --hashdocsettings-path ~/KOReaderSettings/hashdocsettings --data-path ~/koshelf-data
```

### 3. Docsettings

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

**Note:** Unlike KOReader, KoShelf matches books by filename only, since the folder structure reflects the device path (which may differ from your local path). If you have multiple books with the same filename, KoShelf will show an error — use `hashdocsettings` or the default metadata-next-to-book mode instead.

**Usage:**

```bash
koshelf serve --library-path ~/Books --docsettings-path ~/KOReaderSettings/docsettings --data-path ~/koshelf-data
```

## Typical Deployment Setup

One common deployment pattern:

1. **Syncthing sync**: Use [Syncthing](https://syncthing.net/) to sync both your books folder and KoReader settings folder from your e-reader to your server
2. **Books and statistics**: Point to the synced books folder with `--library-path` and to `statistics.sqlite3` in the synced KoReader settings folder with `--statistics-db`
3. **Web server mode**: Run KoShelf in serve mode with `koshelf serve`
4. **Reverse proxy**: Put nginx in front for HTTPS and access control

Example command:

```bash
# Example server command - serves continuously and refreshes data when files change
koshelf serve --library-path ~/syncthing/Books \
              --statistics-db ~/syncthing/KOReaderSettings/statistics.sqlite3 \
              --data-path ~/koshelf-data \
              --port 3000
```

With this setup, each Syncthing update refreshes the site with your latest reading progress, highlights, and statistics.

## Reading Statistics from Multiple Devices

When you read on more than one device, there are three ways to get all reading statistics into KoShelf:

### 1. Sync the database file itself (e.g. Syncthing)

Sync `statistics.sqlite3` between devices with a file sync tool so all devices share one database, and point `--statistics-db` at any synced copy.

This only works with **strictly serial reading**: finish on one device, let the sync complete, then pick up the next device. SQLite files cannot be merged at the file level — if two devices write to their copies before syncing, one version wins (or the sync tool creates a conflict file) and the other device's sessions are silently lost.

### 2. KOReader's built-in statistics sync

The statistics plugin can synchronize through KOReader's cloud storage (WebDAV or Dropbox): each device merges the shared history into its own database, so every synced device converges to the complete reading history. This handles concurrent reading correctly and also shows combined statistics on the devices themselves.

With this setup KoShelf only needs **one** database — point `--statistics-db` at (a synced copy of) any device's `statistics.sqlite3` after it has synced.

### 3. Supply each device's database to KoShelf

If you don't want to sync anything between devices, give KoShelf every device's database directly — pass `-s`/`--statistics-db` multiple times or use a path array in `koshelf.toml`:

```bash
koshelf serve -i ~/Books \
              -s ~/syncthing/Kobo/statistics.sqlite3 \
              -s ~/syncthing/Boox/statistics.sqlite3 \
              --data-path ~/koshelf-data
```

KoShelf merges the databases in memory on every load using the same semantics as KOReader's own sync; the source files are never modified, and overlapping or partially synced databases never double count. The devices themselves won't show combined statistics; only KoShelf does. See [Statistics Database](statistics_database.md#merging-multiple-databases) for the technical merge details.

## User-Contributed Setups

See [Syncthing Setups](syncthing_setups/README.md) for community-contributed guides on how to sync your devices with KoShelf.
