export interface ApiMeta {
    version: string;
    generated_at: string;
}

export interface SiteCapabilities {
    has_books: boolean;
    has_comics: boolean;
    has_reading_data: boolean;
}

export interface SiteResponse {
    meta: ApiMeta;
    title: string;
    language: string;
    capabilities: SiteCapabilities;
}

// ── Reading API response types ──────────────────────────────────────────

export interface ApiResponse<T> {
    data: T;
    meta: { generated_at: string };
}

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
    value: number;
}

export interface ReadingMetricsData {
    metric: string;
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

// ── Static export aggregate types (for client-side scope extraction) ────

export interface ExportSite {
    name: string;
    version: string;
    default_language: string;
    capabilities: SiteCapabilities;
}

export interface ExportReadingSummary {
    all: ReadingSummaryData;
    books: ReadingSummaryData;
    comics: ReadingSummaryData;
}

export interface ExportPeriodsByScope {
    all: ReadingAvailablePeriodsData;
    books: ReadingAvailablePeriodsData;
    comics: ReadingAvailablePeriodsData;
}

export interface ExportReadingPeriods {
    reading_data: {
        week: ExportPeriodsByScope;
        month: ExportPeriodsByScope;
        year: ExportPeriodsByScope;
    };
    completions: {
        month: ExportPeriodsByScope;
        year: ExportPeriodsByScope;
    };
}

export interface ExportDayMetrics {
    reading_time_sec: number;
    pages_read: number;
    sessions: number;
    completions: number;
}

export interface ExportDayMetricsByScope {
    all: ExportDayMetrics;
    books: ExportDayMetrics;
    comics: ExportDayMetrics;
}

export interface ExportMonthMetrics {
    month: string;
    days: Record<string, ExportDayMetricsByScope>;
}
