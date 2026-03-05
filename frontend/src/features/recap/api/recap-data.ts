import { api, type ScopeValue } from '../../../shared/api';

export type RecapScope = ScopeValue;
export type RecapContentType = 'book' | 'comic';

export interface RecapIndexResponse {
    available_years: number[];
    latest_year?: number | null;
}

export interface RecapSummaryResponse {
    total_items: number;
    total_time_seconds: number;
    total_time_days: number;
    total_time_hours: number;
    longest_session_hours: number;
    longest_session_minutes: number;
    average_session_hours: number;
    average_session_minutes: number;
    active_days: number;
    active_days_percentage: number;
    longest_streak: number;
    best_month_name?: string | null;
}

export interface RecapItemResponse {
    item_id?: string | null;
    title: string;
    authors: string[];
    start_date: string;
    end_date: string;
    reading_time: number;
    session_count: number;
    pages_read: number;
    calendar_length_days?: number | null;
    average_speed?: number | null;
    avg_session_duration?: number | null;
    rating?: number | null;
    review_note?: string | null;
    series?: string | null;
    item_cover?: string | null;
    content_type?: RecapContentType | null;
}

export interface RecapMonthResponse {
    month_key: string;
    month_label: string;
    items_finished: number;
    read_time: number;
    items: RecapItemResponse[];
}

export interface RecapShareAssets {
    story_url: string;
    square_url: string;
    banner_url: string;
}

export interface RecapYearResponse {
    year: number;
    summary: RecapSummaryResponse;
    months: RecapMonthResponse[];
    items: RecapItemResponse[];
    share_assets?: RecapShareAssets | null;
}

export async function loadRecapIndex(
    scope: RecapScope,
): Promise<RecapIndexResponse> {
    return api.completions.years.get<RecapIndexResponse>(scope);
}

export async function loadRecapYear(
    scope: RecapScope,
    year: number,
): Promise<RecapYearResponse> {
    return api.completions.years.byKey<RecapYearResponse>(year, scope);
}
