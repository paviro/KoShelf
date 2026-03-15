# KoShelf API Reference

All endpoints are served under `/api/` when running in web server mode.

## Response Envelope

Every successful response is wrapped in an `ApiResponse<T>` envelope:

```json
{
  "data": { ... },
  "meta": {
    "version": "0.1.0",
    "generated_at": "2026-03-14T12:00:00Z"
  }
}
```

Error responses use a separate shape:

```json
{
  "error": {
    "code": "invalid_query",
    "message": "Human-readable description"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `invalid_query` | 400 | Invalid or missing query parameter |
| `invalid_week_key` | 400 | Week key is not a valid Monday date |
| `invalid_month_key` | 400 | Month key is not in YYYY-MM format |
| `invalid_year` | 400 | Year is not a valid 4-digit number |
| `not_found` | 404 | Requested resource does not exist |
| `internal_server_error` | 500 | Server-side error |

---

## Common Parameters

Several parameters are shared across multiple endpoints:

### `scope`

Filters results by content type.

| Value | Description |
|-------|-------------|
| `all` | All content types (default) |
| `books` | Books only |
| `comics` | Comics only |

### `from` / `to`

Date range filter. Both must be provided together or both omitted. Format: `YYYY-MM-DD`. `to` must be >= `from`.

### `tz`

IANA timezone name (e.g. `America/New_York`, `Europe/Berlin`). Controls how date boundaries and period calculations are applied. Defaults to UTC if omitted.

---

## Endpoints

### `GET /api/site`

Returns site metadata and capabilities.

**Parameters:** None

**Response:**

```json
{
  "title": "KoShelf",
  "language": "en_US",
  "capabilities": {
    "has_books": true,
    "has_comics": false,
    "has_reading_data": true
  }
}
```

---

### `GET /api/items`

Returns all library items with optional filtering and sorting.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `scope` | string | No | `all` | Content type filter: `all`, `books`, `comics` |
| `sort` | string | No | `title` | Sort field (see below) |
| `order` | string | No | varies | `asc` or `desc` (default depends on sort field) |

**Sort fields and default order:**

| Sort Value | Default Order | Description |
|------------|---------------|-------------|
| `title` | `asc` | Alphabetical by title |
| `author` | `asc` | Alphabetical by first author |
| `status` | `asc` | By reading status |
| `progress` | `desc` | By progress percentage |
| `rating` | `desc` | By user rating |
| `annotations` | `desc` | By annotation count |
| `last_open_at` | `desc` | By last opened date |

**Response:**

```json
{
  "items": [
    {
      "id": "abc123",
      "title": "Example Book",
      "authors": ["Author Name"],
      "series": { "name": "Series Name", "index": "1" },
      "status": "reading",
      "progress_percentage": 45.2,
      "rating": 4,
      "annotation_count": 12,
      "cover_url": "/assets/covers/abc123.webp",
      "content_type": "book"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique item identifier |
| `title` | string | Item title |
| `authors` | string[] | List of authors |
| `series` | object? | Series info with `name` (string) and optional `index` (string) |
| `status` | string | One of: `reading`, `complete`, `abandoned`, `unknown` |
| `progress_percentage` | number? | Reading progress 0–100 |
| `rating` | number? | User rating (typically 0–5) |
| `annotation_count` | number | Number of annotations |
| `cover_url` | string | Path to cover image |
| `content_type` | string | `book` or `comic` |

---

### `GET /api/items/{id}`

Returns detailed information for a single library item.

**Path Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | string | Yes | Library item ID |

**Query Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `include` | string | No | (none) | Comma-separated list of extra data to include |

**Include tokens:**

| Token | Description |
|-------|-------------|
| `highlights` | Include highlight annotations |
| `bookmarks` | Include bookmark annotations |
| `statistics` | Include reading statistics |
| `completions` | Include completion history |
| `all` | Include everything (supersedes other tokens) |

Unknown tokens return a 400 error. Duplicates and extra whitespace are ignored.

**Response:**

The `item` object extends the list item with additional fields:

| Field | Type | Description |
|-------|------|-------------|
| `language` | string? | Content language |
| `publisher` | string? | Publisher name |
| `description` | string? | Book description (sanitized HTML) |
| `review_note` | string? | User's review / summary note |
| `pages` | number? | Total page count |
| `search_base_path` | string | Base path for external search links |
| `subjects` | string[] | Genres / subjects |
| `identifiers` | object[] | External identifiers (see below) |

Each identifier:

| Field | Type | Description |
|-------|------|-------------|
| `scheme` | string | Identifier type (e.g. `isbn`, `asin`) |
| `value` | string | The identifier value |
| `display_scheme` | string | Human-readable scheme name |
| `url` | string? | Link to external catalog |

**Optional includes:**

`highlights` / `bookmarks` — arrays of annotations:

| Field | Type | Description |
|-------|------|-------------|
| `chapter` | string? | Chapter title |
| `datetime` | string? | Timestamp |
| `pageno` | number? | Page number |
| `text` | string? | Highlighted text |
| `note` | string? | User note |

`statistics`:

| Field | Type | Description |
|-------|------|-------------|
| `item_stats.notes` | number? | Note count |
| `item_stats.last_open_at` | string? | ISO 8601 timestamp |
| `item_stats.highlights` | number? | Highlight count |
| `item_stats.pages` | number? | Page count |
| `item_stats.total_reading_time_sec` | number? | Total reading time in seconds |
| `session_stats.session_count` | number | Number of reading sessions |
| `session_stats.average_session_duration_sec` | number? | Average session length |
| `session_stats.longest_session_duration_sec` | number? | Longest session length |
| `session_stats.last_read_date` | string? | ISO 8601 date |
| `session_stats.reading_speed` | number? | Pages per hour |

`completions`:

| Field | Type | Description |
|-------|------|-------------|
| `entries` | object[] | Completion records |
| `entries[].start_date` | string | ISO 8601 date |
| `entries[].end_date` | string | ISO 8601 date |
| `entries[].reading_time_sec` | number | Time spent reading |
| `entries[].session_count` | number | Number of sessions |
| `entries[].pages_read` | number | Pages read |
| `total_completions` | number | Total times completed |
| `last_completion_date` | string? | ISO 8601 date |

**Status Codes:** 200, 400 (invalid include), 404 (item not found)

---

### `GET /api/reading/summary`

Returns aggregate reading statistics for a time period.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `scope` | string | No | `all` | Content type filter |
| `from` | string | No | — | Start date (YYYY-MM-DD), requires `to` |
| `to` | string | No | — | End date (YYYY-MM-DD), requires `from` |
| `tz` | string | No | UTC | IANA timezone |

**Response:**

```json
{
  "range": {
    "from": "2025-01-01T00:00:00Z",
    "to": "2025-12-31T23:59:59Z",
    "tz": "UTC"
  },
  "overview": {
    "reading_time_sec": 360000,
    "pages_read": 5000,
    "sessions": 200,
    "completions": 15,
    "items_completed": 12,
    "longest_reading_time_in_day_sec": 7200,
    "most_pages_in_day": 120,
    "average_session_duration_sec": 1800,
    "longest_session_duration_sec": 5400
  },
  "streaks": {
    "current": { "days": 5, "start_date": "2025-12-27", "end_date": "2025-12-31" },
    "longest": { "days": 30, "start_date": "2025-03-01", "end_date": "2025-03-30" }
  },
  "heatmap_config": {
    "max_scale_sec": 7200
  }
}
```

---

### `GET /api/reading/metrics`

Returns time-series reading data points grouped by period.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `metric` | string | **Yes** | — | Comma-separated metric names (see below) |
| `group_by` | string | **Yes** | — | Grouping period (see below) |
| `scope` | string | No | `all` | Content type filter |
| `from` | string | No | — | Start date (YYYY-MM-DD), requires `to` |
| `to` | string | No | — | End date (YYYY-MM-DD), requires `from` |
| `tz` | string | No | UTC | IANA timezone |

**Available metrics:**

| Metric | Description |
|--------|-------------|
| `reading_time_sec` | Total reading time in seconds |
| `pages_read` | Total pages read |
| `sessions` | Number of reading sessions |
| `completions` | Number of books completed |
| `average_session_duration_sec` | Average session length |
| `longest_session_duration_sec` | Longest session length |

**Group-by values:**

| Value | Key Format | Description |
|-------|------------|-------------|
| `total` | `all-time` | Single aggregate point |
| `day` | `YYYY-MM-DD` | Daily breakdown |
| `week` | `YYYY-MM-DD` | Weekly (key = Monday date) |
| `month` | `YYYY-MM` | Monthly breakdown |
| `year` | `YYYY` | Yearly breakdown |

**Response:**

```json
{
  "metrics": ["reading_time_sec", "pages_read"],
  "group_by": "day",
  "scope": "all",
  "points": [
    { "key": "2025-12-01", "reading_time_sec": 3600, "pages_read": 50 },
    { "key": "2025-12-02", "reading_time_sec": 1800, "pages_read": 25 }
  ]
}
```

Metric values are flattened directly into each point object.

---

### `GET /api/reading/available-periods`

Returns available time periods that have reading data, useful for building period selectors.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `source` | string | **Yes** | — | Data source: `reading_data` or `completions` |
| `group_by` | string | **Yes** | — | Grouping: `week`, `month`, or `year` |
| `scope` | string | No | `all` | Content type filter |
| `from` | string | No | — | Start date (YYYY-MM-DD), requires `to` |
| `to` | string | No | — | End date (YYYY-MM-DD), requires `from` |
| `tz` | string | No | UTC | IANA timezone |

**Source/group_by constraints:**

- `source=completions` does **not** support `group_by=week`
- All other combinations are valid

**Response:**

```json
{
  "source": "reading_data",
  "group_by": "month",
  "periods": [
    {
      "key": "2025-12",
      "start_date": "2025-12-01T00:00:00Z",
      "end_date": "2025-12-31T23:59:59Z",
      "reading_time_sec": 36000,
      "pages_read": 500
    }
  ],
  "latest_key": "2025-12"
}
```

When `source=completions`, periods contain `completions` (number) instead of `reading_time_sec` / `pages_read`.

---

### `GET /api/reading/calendar`

Returns reading events and statistics for a calendar month.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `month` | string | **Yes** | — | Month in `YYYY-MM` format |
| `scope` | string | No | `all` | Content type filter |
| `tz` | string | No | UTC | IANA timezone |

**Response:**

```json
{
  "month": "2025-12",
  "events": [
    {
      "item_ref": "ref-1",
      "start": "2025-12-01T20:00:00Z",
      "end": "2025-12-01T21:00:00Z",
      "reading_time_sec": 3600,
      "pages_read": 45
    }
  ],
  "items": {
    "ref-1": {
      "title": "Example Book",
      "authors": ["Author Name"],
      "content_type": "book",
      "item_id": "abc123",
      "item_cover": "/assets/covers/abc123.webp"
    }
  },
  "stats_by_scope": {
    "all": { "items_read": 3, "pages_read": 500, "reading_time_sec": 36000, "active_days_percentage": 65.0 },
    "books": { "items_read": 2, "pages_read": 400, "reading_time_sec": 30000, "active_days_percentage": 55.0 },
    "comics": { "items_read": 1, "pages_read": 100, "reading_time_sec": 6000, "active_days_percentage": 20.0 }
  }
}
```

The `items` map is keyed by `item_ref` values used in the `events` array. `item_id` and `item_cover` are present only if the item exists in the library catalog.

---

### `GET /api/reading/completions`

Returns books completed in a time period with detailed per-item statistics.

**Parameters:**

| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `scope` | string | No | `all` | Content type filter |
| `year` | string | No | latest | 4-digit year (mutually exclusive with `from`/`to`) |
| `from` | string | No | — | Start date (YYYY-MM-DD), mutually exclusive with `year` |
| `to` | string | No | — | End date (YYYY-MM-DD), mutually exclusive with `year` |
| `group_by` | string | No | `month` | Grouping: `month` or `none` |
| `include` | string | No | (none) | Comma-separated: `summary`, `share_assets` |
| `tz` | string | No | UTC | IANA timezone |

If neither `year` nor `from`/`to` is provided, defaults to the latest year with completions.

**Response (group_by=month):**

```json
{
  "groups": [
    {
      "key": "2025-03",
      "items_finished": 2,
      "reading_time_sec": 72000,
      "items": [ ... ]
    }
  ]
}
```

**Response (group_by=none):**

```json
{
  "items": [ ... ]
}
```

Each completion item:

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Item title |
| `authors` | string[] | Authors |
| `start_date` | string | ISO 8601 date |
| `end_date` | string | ISO 8601 date |
| `reading_time_sec` | number | Total reading time |
| `session_count` | number | Number of sessions |
| `pages_read` | number | Pages read |
| `calendar_length_days` | number? | Days between start and end |
| `average_speed` | number? | Pages per hour |
| `average_session_duration_sec` | number? | Average session length |
| `rating` | number? | User rating |
| `review_note` | string? | User review |
| `series` | string? | Series name |
| `item_id` | string? | Library item ID (if in catalog) |
| `item_cover` | string? | Cover URL (if in catalog) |
| `content_type` | string? | `book` or `comic` |

**Optional include: `summary`**

| Field | Type | Description |
|-------|------|-------------|
| `total_items` | number | Items completed |
| `total_reading_time_sec` | number | Total reading time |
| `longest_session_duration_sec` | number | Longest single session |
| `average_session_duration_sec` | number | Average session length |
| `active_days` | number | Days with reading activity |
| `active_days_percentage` | number | Percentage of days active (0–100) |
| `longest_streak_days` | number | Longest consecutive reading streak |
| `best_month` | string? | Month with most completions (YYYY-MM) |

**Optional include: `share_assets`**

| Field | Type | Description |
|-------|------|-------------|
| `story_url` | string | URL to story-format share image |
| `square_url` | string | URL to square-format share image |
| `banner_url` | string | URL to banner-format share image |

---

### `GET /api/events/stream`

Server-Sent Events (SSE) stream for real-time data change notifications.

**Parameters:** None

**Behavior:**
- Content-Type: `text/event-stream`
- Event type: `data_changed`
- Keep-alive sent every 15 seconds
- Clients should reconnect on disconnect and reload relevant data when a `data_changed` event is received
