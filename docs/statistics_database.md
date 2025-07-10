# KOReader Statistics Database Overview

## Database Schema

### Core Tables

#### 1. `book` table
Stores book metadata and aggregate statistics.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Unique book identifier |
| `title` | TEXT | Book title |
| `authors` | TEXT | Book authors (or "N/A" if empty) |
| `md5` | TEXT | Partial MD5 checksum of the book file |
| `pages` | INTEGER | Total page count when last read |
| `last_open` | INTEGER | Unix timestamp of last access |
| `total_read_time` | INTEGER | Total seconds spent reading |
| `total_read_pages` | INTEGER | Total pages read (including re-reads) |
| `highlights` | INTEGER | Number of highlights |
| `notes` | INTEGER | Number of notes |
| `series` | TEXT | Book series info |
| `language` | TEXT | Book language |

**Unique Index**: `(title, authors, md5)`

#### 2. `page_stat_data` table
Raw reading session data.

| Column | Type | Description |
|--------|------|-------------|
| `id_book` | INTEGER | Foreign key to book.id |
| `page` | INTEGER | Page number at time of reading |
| `start_time` | INTEGER | Unix timestamp when reading started |
| `duration` | INTEGER | Seconds spent on this page |
| `total_pages` | INTEGER | Book's page count at time of reading |

**Unique Constraint**: `(id_book, page, start_time)`

#### 3. `page_stat` view
Automatically rescales historical page numbers to current book layout.

This is a **VIEW**, not a table. It dynamically adjusts page numbers based on the current book's page count.

#### 4. `numbers` table
Helper table containing integers 1-1000 for the rescaling view.

## Key Concepts

### Page Number Rescaling

The most critical concept in the database is **page rescaling**. Books can change their page count when:
- Font size changes
- Margins are adjusted
- Screen orientation changes
- Different rendering settings applied

The database handles this by:
1. Storing `total_pages` at the time of each reading session
2. Using the `page_stat` view to rescale historical page numbers

**Rescaling Formula:**
```lua
-- First page after rescaling
first_page = ((page - 1) * current_pages) / total_pages + 1

-- Last page after rescaling  
last_page = max(first_page, (page * current_pages) / total_pages)
```

### Duration Tracking

Reading durations are tracked with these constraints:
- **min_sec** (default: 5 seconds): Minimum time to count a page as "read"
- **max_sec** (default: 120 seconds): Maximum time counted per page
- Sessions are automatically split when page turns occur
- Zero-duration entries indicate the current reading position

## Common Queries

### Get Total Reading Time (Capped)
```sql
SELECT count(*), sum(durations)
FROM (
    SELECT min(sum(duration), 120) AS durations  -- 120 is max_sec
    FROM page_stat
    WHERE id_book = ?
    GROUP BY page
);
```

### Get Total Reading Time (Uncapped)
```sql
SELECT sum(duration), count(DISTINCT page)
FROM page_stat
WHERE id_book = ?;
```

### Get Reading Sessions by Day
```sql
SELECT date(start_time, 'unixepoch', 'localtime') as day,
       count(DISTINCT page) as unique_pages,
       sum(duration) as total_time
FROM page_stat
WHERE id_book = ?
GROUP BY day
ORDER BY day DESC;
```

### Get Books Read in Time Period
```sql
SELECT book.title, 
       count(DISTINCT page_stat.page) as pages_read,
       sum(page_stat.duration) as time_spent
FROM page_stat
JOIN book ON book.id = page_stat.id_book
WHERE page_stat.start_time BETWEEN ? AND ?
GROUP BY book.id
ORDER BY time_spent DESC;
```

### Get Reading Progress
```sql
SELECT page,
       sum(duration) as time_on_page,
       count(*) as visits
FROM page_stat
WHERE id_book = ?
GROUP BY page
ORDER BY page;
````

## Implementation Guidelines

### 1. Always Use Views for Queries
- Use `page_stat` view for all queries unless you specifically need raw historical data
- The view handles page number rescaling automatically

### 2. Duration Interpretation
- All durations are stored in seconds
- Multiple reading sessions on the same page create separate rows
- Zero duration indicates current reading position (not yet turned page)

### 3. Page Counting Methods
- **Unique pages**: `COUNT(DISTINCT page)`
- **Total page views**: `COUNT(*)`
- **Reading progress**: Compare unique pages to book.pages

### 4. Time Calculation Considerations
- **Capped times**: Better for averages and estimates
- **Uncapped times**: Show true total reading time
- Use capped queries for "time per page" calculations
- Use uncapped for "total time spent" displays

### 5. Book Identification
- Books are uniquely identified by `(title, authors, md5)` tuple
- Empty authors are stored as "N/A" (legacy: empty string "")
- MD5 is a partial checksum for file identification

### 6. Handling Page Count Changes
- Always store current `total_pages` when inserting new stats
- The view automatically handles rescaling for queries
- Historical data preserves original page numbers in `page_stat_data`

## Database Migrations

The database uses `PRAGMA user_version` for schema versioning:
- Current version: 20221111
- Migrations handle legacy data format conversions
- WAL mode is enabled on supported devices

## Sync Considerations

When syncing between devices:
1. Books are matched by `(title, authors, md5)`
2. Page statistics are merged, keeping maximum duration for conflicts
3. Deleted books are tracked through cached database comparison
4. `last_open` timestamps determine most recent device access