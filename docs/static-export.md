# Static Export Structure

When using `koshelf export`, the following directory structure is generated:

```
site/
├── index.html              # React app shell (routes handled via HashRouter)
├── manifest.json           # PWA manifest
├── assets/
│   ├── covers/             # Optimized cover images
│   │   ├── <item-id>.webp
│   │   └── ...
│   ├── files/              # Optional original item files (only with --include-files)
│   │   ├── <item-id>.epub
│   │   ├── <item-id>.cbz
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
    │   ├── books.json          # Books only (filtered subset)
    │   ├── comics.json         # Comics only (filtered subset)
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

`assets/files/` is generated only when `--include-files` (or `[output].include_files = true`) is enabled. Because it copies original item files, export size can grow substantially.

In `serve` mode, the equivalent `/assets/files/**` path is a runtime asset route. If authentication is enabled, it is protected by the auth middleware like other `/assets/**` routes.

Static exports do not provide built-in authentication. If you export files, use host-level protection (for example: reverse-proxy auth, private bucket/CDN rules, or password-gated hosting) when those downloads should not be publicly accessible.
