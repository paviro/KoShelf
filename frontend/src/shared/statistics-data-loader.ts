import { api, type ScopeValue } from './api';

export type StatisticsScope = ScopeValue;

export interface DailyActivityEntry {
    date: string;
    read_time: number;
    pages_read: number;
}

export interface ActivityConfig {
    max_scale_seconds: number | null;
}

export interface YearlyActivitySummary {
    completed_count: number;
}

export interface StatisticsWeekResponse {
    week_key: string;
    start_date: string;
    end_date: string;
    read_time: number;
    pages_read: number;
    avg_pages_per_day: number;
    avg_read_time_per_day: number;
    longest_session_duration: number | null;
    average_session_duration: number | null;
}

export interface StatisticsIndexWeek {
    week_key: string;
    start_date: string;
    end_date: string;
}

export interface StatisticsIndexResponse {
    available_years: number[];
    available_weeks: StatisticsIndexWeek[];
    overview: {
        total_read_time: number;
        total_page_reads: number;
        longest_read_time_in_day: number;
        most_pages_in_day: number;
        average_session_duration: number | null;
        longest_session_duration: number | null;
        total_completions: number;
        books_completed: number;
        most_completions: number;
    };
    streaks: {
        longest: {
            days: number;
            start_date: string | null;
            end_date: string | null;
        };
        current: {
            days: number;
            start_date: string | null;
            end_date: string | null;
        };
    };
    heatmap_config: ActivityConfig;
}

export interface StatisticsYearResponse {
    year: number;
    summary: YearlyActivitySummary;
    daily_activity: DailyActivityEntry[];
    monthly_aggregates: Array<{
        month_key: string;
        read_time: number;
        pages_read: number;
        active_days: number;
    }>;
    config: ActivityConfig;
}

export async function loadStatisticsIndex(
    scope: StatisticsScope,
): Promise<StatisticsIndexResponse> {
    return api.statistics.get<StatisticsIndexResponse>(scope);
}

export async function loadStatisticsWeek(
    scope: StatisticsScope,
    weekKey: string,
): Promise<StatisticsWeekResponse> {
    return api.statistics.weeks.get<StatisticsWeekResponse>(weekKey, scope);
}

export async function loadStatisticsYear(
    scope: StatisticsScope,
    year: number,
): Promise<StatisticsYearResponse> {
    return api.statistics.years.get<StatisticsYearResponse>(year, scope);
}
