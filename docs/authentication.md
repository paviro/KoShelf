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

Protected routes include `/api/**` (except `GET /api/site` and `POST /api/auth/login`) and runtime assets under `/assets/**` (including `/assets/covers/**`, `/assets/files/**`, and `/assets/recap/**`). Shell assets under `/core/**` remain public.

## Reverse-Proxy Authentication

KoShelf can delegate authentication to a reverse proxy that already authenticates users (for example Home Assistant ingress or an SSO gateway). Proxy authentication is an addition to password authentication, not a replacement, so `--enable-auth` is required.

```bash
koshelf serve -i ~/Library --data-path ~/koshelf-data --enable-auth \
  --bind-address 127.0.0.1 --auth-proxy-peers 127.0.0.1/32
```

Requests from a peer listed in `--auth-proxy-peers` (`KOSHELF_AUTH_PROXY_PEERS`, TOML `auth_proxy_peers`) may send:

- `X-KoShelf-User`: the authenticated user name (1-256 characters from `A-Z a-z 0-9 @ . _ -`). The request is treated as authenticated.
- `X-Forwarded-Prefix`: the sub-path KoShelf is served under (for example `/koshelf` or `/api/hassio_ingress/<token>`). Segments may only contain `A-Z a-z 0-9 _ -`; no trailing slash.

Rules and caveats:

- The peer list is matched against the TCP peer address only and is independent of `--trusted-proxies`. List only the proxy itself, never client networks.
- **The proxy must remove or overwrite `X-KoShelf-User` and `X-Forwarded-Prefix` on every request it forwards.** Otherwise any client that can reach the proxy can impersonate a user.
- Requests carrying either header from any other peer, or carrying a header more than once, are rejected with `403`.
- Requests from a listed peer without `X-KoShelf-User` (and all direct requests) still require the KoShelf password.
- Proxy-authenticated requests cannot use `/api/auth/**` and never receive a session cookie, so a proxy login cannot be turned into a standalone KoShelf session. Password and session management are hidden in the UI.
- Proxy-authenticated responses allow same-origin framing (`frame-ancestors 'self'`) so KoShelf can be embedded in panels such as Home Assistant.
- Bind KoShelf to `127.0.0.1` (or a private network) when the proxy runs alongside it, so clients cannot bypass the proxy.

## Write Request Protection

All non-`GET`/`HEAD`/`OPTIONS` requests must include `X-KoShelf-Request: 1` and are rejected if the browser reports a `Sec-Fetch-Site` other than `same-origin`. This prevents cross-site request forgery. The web UI sends the header automatically; scripts calling write endpoints must add it.

Authentication is not available in static export mode (`koshelf export`), so any hosted static files are governed by your hosting setup.
