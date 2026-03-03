# KoShelf React Frontend (Scaffold)

This workspace is the initial React + Vite shell for the staged migration.

Current decisions reflected here:

- Router mode: `HashRouter`
- Data fetching: TanStack Query
- Target mount in serve mode: `/`

## Commands

From repository root:

```bash
npm --prefix frontend install
npm --prefix frontend run dev
```

## Vite + Rust backend dev loop

Use this when you want frontend hot-reload without rebuilding/restarting Rust for every UI edit.

1. Start the KoShelf backend (API/static server) on `http://localhost:3000`.
    - Optional for faster Rust rebuilds while using Vite: set `KOSHELF_SKIP_REACT_BUILD=1`.
2. In another terminal run:

```bash
npm --prefix frontend run dev
```

The Vite dev server proxies backend routes (`/api`, `/assets`) to KoShelf.

Notes:

- In Vite dev mode, the React app defaults to `internal` server mode so requests go to `/api/*`.
- You can target a different backend URL by setting `KOSHELF_DEV_BACKEND_URL`:

```bash
KOSHELF_DEV_BACKEND_URL=http://localhost:4000 npm --prefix frontend run dev
```

Build and typecheck:

```bash
npm --prefix frontend run typecheck
npm --prefix frontend run build
```

This scaffold currently contains route placeholders and basic `/api/site` loading only.
