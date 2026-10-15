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

###### A reading companion powered by KOReader metadata — browse your library, highlights, annotations, and reading statistics from a web dashboard.

</div>

## Features

- 📚 **Library Overview (Books + Comics)**: Browse currently reading, completed, and unread books, comics, and other documents
- 🗂️ **Metadata Entries**: Browse KOReader reading data even without the original document, including PDF and DjVu sidecars; edit metadata, covers, and annotations when writeback is enabled
- 🗄️ **Library Archives**: Preserve reading data and covers after both the document and sidecar disappear; returning sources restore the entry automatically ([details](docs/configuration.md#library-preservation))
- 🎨 **Modern UI**: Clean design powered by Tailwind CSS with readable typography and responsive layout
- 📖 **In-App Reader (EPUB/FB2/MOBI/CBZ)**: Open supported files directly in KoShelf with keyboard navigation, progress scrubber controls, and inline highlight notes
- 📝 **Annotations, Highlights & Ratings**: KOReader highlights, notes, ratings, and review notes together on each item’s details page, including page-numbered PDF annotations
- ✏️ **Metadata Writeback**: Edit annotations, review notes, ratings, reading status, custom metadata, and covers in KoShelf, with revision checks to prevent overwriting external changes and support for recoverable KOReader backups (serve mode, opt-in)
- 📊 **Reading Statistics**: Track your reading habits with detailed statistics including reading time, pages read, customizable activity heatmaps, and weekly breakdowns
- 📅 **Reading Calendar**: Monthly calendar view showing your reading activity with items read on each day and monthly statistics
- 🎉 **Yearly Recap**: Celebrate your reading year with a timeline of completions, monthly summaries (finished items, hours read), and rich per-item details
- 📈 **Per-Item Statistics**: Detailed statistics for each item including session count, average session duration, reading speed, last read date, and a page-level reading activity heatmap
- 🔍 **Search & Filter**: Search through your library by title, author, or series, with filters for reading status
- ⬇️ **Original File Downloads**: Download documents that are present, including formats KoShelf cannot parse or render; static exports support downloads with `--include-files`
- 🔐 **Optional Authentication**: Use password authentication with session management, or delegate authentication to explicitly trusted reverse proxies such as Home Assistant ingress or an SSO gateway ([setup](docs/authentication.md))
- 🚀 **Static Site**: Generates a complete static website you can host anywhere
- 🖥️ **Server Mode**: Built-in web server with live file watching, configurable bind address, and sub-path hosting through a trusted reverse proxy
- 📱 **Responsive**: Optimized for desktop, tablet, and mobile with adaptive grid layouts

## Screenshots

![Library overview](https://github.com/user-attachments/assets/ad096bc9-c53a-40eb-9de9-06085e854a26)
![Book details](https://github.com/user-attachments/assets/44113be0-aa19-4018-b864-135ddb067a9d)
![Reading calendar](https://github.com/user-attachments/assets/a4ac51f1-927e-463d-b2d6-72c29fdc4323)
![Recap](https://github.com/user-attachments/assets/9558eea9-dee1-4b0a-adac-1bc0157f0181)

## Quick Start

### Install

**Home Assistant** — [One-click install](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

[![Open your Home Assistant instance and show the dashboard of an add-on.](https://my.home-assistant.io/badges/supervisor_addon.svg)](https://my.home-assistant.io/redirect/supervisor_addon/?addon=5d189d71_koshelf&repository_url=https%3A%2F%2Fgithub.com%2Fpaviro%2Fhome-assistant-addons)

**Docker Compose** — Community-maintained image via [koshelf-docker](https://github.com/DevTigro/koshelf-docker). See the [Installation Guide](docs/installation.md) for a sample `docker-compose.yml`.

**Prebuilt binaries** — Download from the [releases page](https://github.com/paviro/koshelf/releases) (Linux GNU/musl, Windows MSVC, and macOS for x86_64 and ARM64).

For detailed installation instructions (building from source, first-time CLI guide, Windows Defender notes), see the [Installation Guide](docs/installation.md).

### Usage

```bash
# Start a web server (live file watching, requires --data-path)
koshelf serve -i ~/Library --data-path ~/koshelf-data

# Generate a static site
koshelf export ~/my-reading-site -i ~/Library

# Generate a static site with downloadable original files
koshelf export ~/my-reading-site -i ~/Library --include-files
```

For all subcommands, options, environment variables, and examples, see the [Configuration Guide](docs/configuration.md).

## Documentation

| Guide | Description |
|-------|-------------|
| [Installation](docs/installation.md) | Home Assistant, Docker, prebuilt binaries, building from source |
| [Configuration](docs/configuration.md) | Subcommands, CLI options, environment variables, config file, examples |
| [KoReader Setup](docs/koreader-setup.md) | Metadata storage options (sdr/hashdocsettings/docsettings), deployment setup, reading statistics from multiple devices |
| [Authentication](docs/authentication.md) | Password and trusted-proxy authentication, sub-path hosting, write protection |
| [Supported Data](docs/supported-data.md) | Supported formats and extracted metadata fields |
| [Stable Page Metadata](docs/stable-page-metadata.md) | KOReader stable page metadata & synthetic page scaling |
| [Static Export](docs/static-export.md) | Generated site directory structure |
| [API Reference](docs/API.md) | REST API endpoints, parameters, and response schemas |
| [Releasing](docs/RELEASING.md) | Release tags, artifacts, signing, notarization, checksums, and attestations |
| [Syncthing Setups](docs/syncthing_setups/README.md) | Community-contributed device sync guides |

## Development

Enable the repository's Git hooks after cloning:

```bash
git config core.hooksPath .githooks
```

Commit messages must use the `type(scope): description` format, for example
`feat(reader): add keyboard navigation`. Add `!` after the scope for a breaking
change, for example `feat(api)!: remove deprecated endpoint`.

## Credits

Design and feature inspiration comes from [KoInsight](https://github.com/GeorgeSG/KoInsight), an excellent alternative that focuses on reading statistics and can also act as a KOReader sync server.

The calendar view is powered by [EventCalendar](https://github.com/vkurko/calendar), a lightweight JavaScript event calendar library.

## Disclaimer

This was built for personal use and relies heavily on AI-generated code. While I've tested everything and use it daily, I take no responsibility for any issues you might encounter. Use at your own risk.
