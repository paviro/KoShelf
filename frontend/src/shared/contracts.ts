export interface SiteCapabilities {
    has_books: boolean;
    has_comics: boolean;
    has_reading_data: boolean;
    has_files?: boolean;
    auth_enabled: boolean;
    has_writeback?: boolean;
}

export interface PasswordPolicy {
    min_chars: number;
}

export interface SiteData {
    title: string;
    language: string;
    capabilities: SiteCapabilities;
    authenticated?: boolean;
    password_policy?: PasswordPolicy;
    version?: string;
    generated_at?: string;
}

export interface SessionInfo {
    id: string;
    user_agent?: string | null;
    browser: string;
    os: string;
    last_seen_ip?: string | null;
    created_at: string;
    last_seen_at: string;
    is_current: boolean;
}

// ── Response envelope ─────────────────────────────────────────────────────

export interface ApiResponse<T> {
    data: T;
    meta: { version: string; generated_at: string };
}

// ── Reading API response types ──────────────────────────────────────────

export interface ResolvedRange {
    from: string;
    to: string;
    tz: string;
}

export interface ReadingOverview {
    reading_time_sec: number;
    pages_read: number;
    sessions: number;
    completions: number;
    items_completed: number;
    longest_reading_time_in_day_sec: number;
    most_pages_in_day: number;
    average_session_duration_sec?: number | null;
    longest_session_duration_sec?: number | null;
}

export interface StreakData {
    days: number;
    start_date?: string | null;
    end_date?: string | null;
}

export interface ReadingStreaks {
    current: StreakData;
    longest: StreakData;
}

export interface HeatmapConfig {
    max_scale_sec?: number | null;
}

export interface ReadingSummaryData {
    range: ResolvedRange;
    overview: ReadingOverview;
    streaks: ReadingStreaks;
    heatmap_config: HeatmapConfig;
}

export interface MetricPoint {
    key: string;
    [metric: string]: string | number;
}

export interface ReadingMetricsData {
    metrics: string[];
    group_by: string;
    scope: string;
    points: MetricPoint[];
}

export interface PeriodEntry {
    key: string;
    start_date: string;
    end_date: string;
    reading_time_sec?: number | null;
    pages_read?: number | null;
    completions?: number | null;
}

export interface ReadingAvailablePeriodsData {
    source: string;
    group_by: string;
    periods: PeriodEntry[];
    latest_key?: string | null;
}

export interface ReadingCalendarEvent {
    item_ref: string;
    start: string;
    end?: string | null;
    reading_time_sec: number;
    pages_read: number;
}

export interface CalendarItemRef {
    title: string;
    authors: string[];
    content_type: 'book' | 'comic';
    item_id?: string | null;
    item_cover?: string | null;
}

export interface CalendarScopeStats {
    items_read: number;
    pages_read: number;
    reading_time_sec: number;
    active_days_percentage: number;
}

export interface ReadingCalendarData {
    month: string;
    events: ReadingCalendarEvent[];
    items: Record<string, CalendarItemRef>;
    stats_by_scope: {
        all: CalendarScopeStats;
        books: CalendarScopeStats;
        comics: CalendarScopeStats;
    };
}

export interface CompletionItem {
    title: string;
    authors: string[];
    start_date: string;
    end_date: string;
    reading_time_sec: number;
    session_count: number;
    pages_read: number;
    calendar_length_days?: number | null;
    average_speed?: number | null;
    average_session_duration_sec?: number | null;
    rating?: number | null;
    review_note?: string | null;
    series?: string | null;
    item_id?: string | null;
    item_cover?: string | null;
    content_type?: 'book' | 'comic' | null;
}

export interface CompletionGroup {
    key: string;
    items_finished: number;
    reading_time_sec: number;
    items: CompletionItem[];
}

export interface CompletionsSummary {
    total_items: number;
    total_reading_time_sec: number;
    longest_session_duration_sec: number;
    average_session_duration_sec: number;
    active_days: number;
    active_days_percentage: number;
    longest_streak_days: number;
    best_month?: string | null;
}

export interface CompletionsShareAssets {
    story_url: string;
    square_url: string;
    banner_url: string;
}

export interface ReadingCompletionsData {
    groups?: CompletionGroup[] | null;
    items?: CompletionItem[] | null;
    summary?: CompletionsSummary | null;
    share_assets?: CompletionsShareAssets | null;
}

// ── Library types (shared) ───────────────────────────────────────────────

export type LibraryContentType = 'book' | 'comic';
export type LibraryStatus = 'reading' | 'complete' | 'abandoned' | 'unknown';

export interface LibrarySeries {
    name: string;
    index?: string | null;
}

export interface LibraryListItem {
    id: string;
    title: string;
    authors: string[];
    series?: LibrarySeries | null;
    status: LibraryStatus;
    progress_percentage?: number | null;
    rating?: number | null;
    annotation_count?: number;
    cover_url: string;
    content_type: LibraryContentType;
}

export interface ExternalIdentifier {
    scheme: string;
    value: string;
    display_scheme: string;
    url?: string | null;
}

export interface LibraryDetailItem {
    id: string;
    title: string;
    authors: string[];
    series?: LibrarySeries | null;
    status: LibraryStatus;
    progress_percentage?: number | null;
    rating?: number | null;
    cover_url: string;
    content_type: LibraryContentType;
    format: string;
    language?: string | null;
    publisher?: string | null;
    description?: string | null;
    review_note?: string | null;
    pages?: number | null;
    search_base_path: string;
    subjects: string[];
    identifiers: ExternalIdentifier[];
    has_metadata?: boolean;
}

export interface LibraryReaderPresentation {
    font_face?: string | null;
    font_size_pt?: number | null;
    line_spacing_percentage?: number | null;
    horizontal_margins?: [number, number] | null;
    top_margin?: number | null;
    bottom_margin?: number | null;
    embedded_fonts?: boolean | null;
    hyphenation?: boolean | null;
    floating_punctuation?: boolean | null;
    word_spacing?: [number, number] | null;
}

export interface LibraryAnnotation {
    id: string;
    chapter?: string | null;
    datetime?: string | null;
    pageno?: number | null;
    text?: string | null;
    note?: string | null;
    pos0?: string | null;
    pos1?: string | null;
    color?: string | null;
    drawer?: string | null;
}

export interface LibraryCompletionEntry {
    start_date: string;
    end_date: string;
    reading_time_sec: number;
    session_count: number;
    pages_read: number;
}

export interface LibraryCompletions {
    entries: LibraryCompletionEntry[];
    total_completions: number;
    last_completion_date?: string | null;
}

export interface LibraryItemStats {
    notes?: number | null;
    last_open_at?: string | null;
    highlights?: number | null;
    bookmarks?: number | null;
    pages?: number | null;
    total_reading_time_sec?: number | null;
}

export interface LibrarySessionStats {
    session_count: number;
    average_session_duration_sec?: number | null;
    longest_session_duration_sec?: number | null;
    last_read_date?: string | null;
    reading_speed?: number | null;
}

export interface LibraryDetailStatistics {
    item_stats?: LibraryItemStats | null;
    session_stats?: LibrarySessionStats | null;
}

export interface LibraryListData {
    items: LibraryListItem[];
}

export interface LibraryDetailData {
    item: LibraryDetailItem;
    highlights?: LibraryAnnotation[] | null;
    bookmarks?: LibraryAnnotation[] | null;
    statistics?: LibraryDetailStatistics | null;
    completions?: LibraryCompletions | null;
    reader_presentation?: LibraryReaderPresentation | null;
}

// ── PageActivity ─────────────────────────────────────────────────────────

export interface PageActivityPage {
    page: number;
    total_duration: number;
    read_count: number;
}

export interface PageActivityAnnotation {
    page: number;
    kind: 'highlight' | 'bookmark' | 'note';
}

export interface PageActivityCompletion {
    index: number;
    start_date: string;
    end_date: string;
}

export interface PageActivityChapter {
    title: string;
    page: number;
}

export interface PageActivityData {
    total_pages: number;
    pages: PageActivityPage[];
    annotations: PageActivityAnnotation[];
    completions: PageActivityCompletion[];
    chapters: PageActivityChapter[];
}

// ── Static export types ──────────────────────────────────────────────────

export interface ExportSite {
    name: string;
    version: string;
    generated_at: string;
    default_language: string;
    capabilities: SiteCapabilities;
}

export interface ExportReadingPeriods {
    reading_data: {
        week: ReadingAvailablePeriodsData;
        month: ReadingAvailablePeriodsData;
        year: ReadingAvailablePeriodsData;
    };
    completions: {
        month: ReadingAvailablePeriodsData;
        year: ReadingAvailablePeriodsData;
    };
}

export interface ExportDayMetrics {
    reading_time_sec: number;
    pages_read: number;
    sessions: number;
    completions: number;
    active_days: number;
    longest_session_duration_sec: number;
    average_session_duration_sec: number;
}

export interface ExportMonthMetrics {
    month: string;
    days: Record<string, ExportDayMetrics>;
}

export interface ExportPageActivityData extends PageActivityData {
    by_completion: Record<string, PageActivityPage[]>;
}
