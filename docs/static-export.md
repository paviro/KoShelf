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
