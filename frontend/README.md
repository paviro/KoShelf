# KoShelf Frontend

React + Vite frontend for KoShelf. This is the UI runtime used by the Rust app in both serve and static output modes.

## Runtime Model

- Router: `HashRouter` (routes resolve under `/#/...`)
- Server state: TanStack Query
- Mode-aware data loading in `src/shared/api.ts`:
  - `internal` mode: fetch from `/api/**`
  - `external` mode: fetch from `/data/**`
- Runtime updates:
  - `internal` mode listens to `/api/events/stream` and invalidates query caches on snapshot updates
  - `external` mode polls `/data/site.json` and invalidates when `generated_at` changes

## Route Coverage

- `/#/books` and `/#/books/:id`
- `/#/comics` and `/#/comics/:id`
- `/#/statistics` and `/#/statistics/:scope`
- `/#/calendar`
- `/#/recap`, `/#/recap/:year`, `/#/recap/:year/:scope`

## Folder Layout

```text
frontend/
  src/
    app/        # app shell + top-level routing composition
    features/   # feature slices (statistics, calendar, library, recap)
    shared/     # cross-feature contracts, API bridge, UI primitives, utilities
    styles/     # global app/calendar styles
```

Feature folders use this baseline shape where relevant:

- `routes/`: route entry components
- `sections/`: page composition blocks
- `components/`: feature-local reusable UI
- `hooks/`, `model/`, `api/`, `lib/`: feature internals

## Architecture Conventions

- Keep `src/app` focused on app wiring (shell, route table, bootstrapping).
- Keep feature/domain logic under `src/features/<feature>`.
- Use `src/shared/ui` only for truly cross-feature UI primitives.
- Use `src/shared/lib` only for cross-feature non-UI utilities.
- Prefer Lucide icons via `react-icons/lu` for new shared/app-level icon usage.

## Commands

Run from repo root:

```bash
npm --prefix frontend install
npm --prefix frontend run dev
```

## Recommended Dev Loop (Vite + Rust backend)

Use this for fast UI iteration:

1. Start KoShelf backend on `http://localhost:3000`
2. In another terminal run:

```bash
npm --prefix frontend run dev
```

Vite proxies:

- `/api` -> backend target
- `/assets` -> backend target

The backend target defaults to `http://localhost:3000`. Override with:

```bash
KOSHELF_DEV_BACKEND_URL=http://localhost:4000 npm --prefix frontend run dev
```

In Vite dev mode, the app defaults to `internal` server mode so requests go to `/api/*`.

## Rust Build Integration

`cargo build` / `cargo run` triggers `build.rs`, which builds this frontend and embeds the resulting assets into the binary.

Useful environment flags:

- `KOSHELF_SKIP_REACT_BUILD=1`: skip frontend build in Rust build pipeline
- `KOSHELF_SKIP_NPM_INSTALL=1`: disable automatic `npm install`/`npm ci` in build script
