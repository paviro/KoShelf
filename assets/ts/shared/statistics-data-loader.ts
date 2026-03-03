import { api, type ScopeValue } from './api.js';

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

export interface YearlyActivityResponse {
    data: DailyActivityEntry[];
    config: ActivityConfig;
    summary: YearlyActivitySummary;
}

export interface StatisticsWeekData {
    start_date: string;
    end_date: string;
    read_time: number;
    pages_read: number;
    avg_pages_per_day: number;
    avg_read_time_per_day: number;
    longest_session_duration: number | null;
    average_session_duration: number | null;
}

export interface StatisticsWeekResponse extends StatisticsWeekData {
    week_key: string;
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

const yearlyActivityCache = new Map<string, Promise<YearlyActivityResponse>>();
const statisticsWeekCache = new Map<string, Promise<StatisticsWeekResponse>>();
const statisticsIndexCache = new Map<string, Promise<StatisticsIndexResponse>>();
const statisticsYearCache = new Map<string, Promise<StatisticsYearResponse>>();

function normalizeScope(scope: string): StatisticsScope {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}

function yearlyActivityCacheKey(scope: StatisticsScope, year: number): string {
    return `${scope}::${year}`;
}

function statisticsWeekCacheKey(scope: StatisticsScope, weekKey: string): string {
    return `${scope}::${weekKey}`;
}

function statisticsYearCacheKey(scope: StatisticsScope, year: number): string {
    return `${scope}::${year}`;
}

async function fetchStatisticsIndex(scope: StatisticsScope): Promise<StatisticsIndexResponse> {
    return api.statistics.get<StatisticsIndexResponse>(scope);
}

async function fetchStatisticsWeek(
    scope: StatisticsScope,
    weekKey: string,
): Promise<StatisticsWeekResponse> {
    return api.statistics.weeks.get<StatisticsWeekResponse>(weekKey, scope);
}

async function fetchStatisticsYear(
    scope: StatisticsScope,
    year: number,
): Promise<StatisticsYearResponse> {
    return api.statistics.years.get<StatisticsYearResponse>(year, scope);
}

export async function loadStatisticsIndex(scope: string): Promise<StatisticsIndexResponse> {
    const normalizedScope = normalizeScope(scope);
    let request = statisticsIndexCache.get(normalizedScope);

    if (!request) {
        request = fetchStatisticsIndex(normalizedScope);
        statisticsIndexCache.set(normalizedScope, request);
    }

    try {
        return await request;
    } catch (error) {
        statisticsIndexCache.delete(normalizedScope);
        throw error;
    }
}

export async function loadStatisticsWeek(
    scope: string,
    weekKey: string,
): Promise<StatisticsWeekResponse> {
    const normalizedScope = normalizeScope(scope);
    const key = statisticsWeekCacheKey(normalizedScope, weekKey);
    let request = statisticsWeekCache.get(key);

    if (!request) {
        request = fetchStatisticsWeek(normalizedScope, weekKey);
        statisticsWeekCache.set(key, request);
    }

    try {
        return await request;
    } catch (error) {
        statisticsWeekCache.delete(key);
        throw error;
    }
}

export async function loadStatisticsYear(
    scope: string,
    year: number,
): Promise<StatisticsYearResponse> {
    const normalizedScope = normalizeScope(scope);
    const key = statisticsYearCacheKey(normalizedScope, year);
    let request = statisticsYearCache.get(key);

    if (!request) {
        request = fetchStatisticsYear(normalizedScope, year);
        statisticsYearCache.set(key, request);
    }

    try {
        return await request;
    } catch (error) {
        statisticsYearCache.delete(key);
        throw error;
    }
}

export async function loadYearlyActivity(
    scope: string,
    year: number,
): Promise<YearlyActivityResponse> {
    const normalizedScope = normalizeScope(scope);
    const key = yearlyActivityCacheKey(normalizedScope, year);
    let request = yearlyActivityCache.get(key);

    if (!request) {
        request = loadStatisticsYear(normalizedScope, year).then((response) => {
            return {
                data: response.daily_activity,
                config: response.config,
                summary: {
                    completed_count: response.summary.completed_count,
                },
            };
        });

        yearlyActivityCache.set(key, request);
    }

    try {
        return await request;
    } catch (error) {
        yearlyActivityCache.delete(key);
        throw error;
    }
}

export function clearYearlyActivityCache(): void {
    yearlyActivityCache.clear();
    statisticsYearCache.clear();
}
