# Authentication

Authentication is optional and only applies in serve mode.

## Setup

1. Start server with auth enabled:
   ```bash
   koshelf serve -i ~/Library --data-path /path/to/runtime-data --enable-auth
   ```
2. On first run, KoShelf generates a password and prints it once.
3. Rotate password anytime via:
   ```bash
   koshelf set-password --data-path /path/to/runtime-data --overwrite
   ```

## `set-password` Command

Use `set-password` to initialize, rotate, or replace the serve-mode password:

```bash
koshelf set-password [--data-path <PATH>] [--password <VALUE> | --random] [--overwrite]
```

- Without `--password`, KoShelf prompts interactively
- With `--random`, KoShelf generates a password and prints it once
- `--password` and `--random` are mutually exclusive
- Password length must be 8-1024 characters
- Data path resolution order: `--data-path` > `KOSHELF_DATA_PATH` > config file
- Without `--overwrite`, command is idempotent (no-op if a password already exists)

### Examples

```bash
# Prompt interactively and replace existing password
koshelf set-password --data-path ~/koshelf-data --overwrite

# Set explicit password (avoid shell history for sensitive values)
koshelf set-password --data-path ~/koshelf-data --password 'correct horse battery staple' --overwrite

# Generate a random password and print it once
koshelf set-password --data-path ~/koshelf-data --random --overwrite
```

## Protected Routes

Protected routes include `/api/**` (except `GET /api/site` and `POST /api/auth/login`) and runtime assets under `/assets/**`. Shell assets under `/core/**` remain public.
