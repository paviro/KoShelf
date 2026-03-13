import { api, type ScopeValue } from '../../../shared/api';
import type {
    HeatmapConfig,
    MetricPoint,
    ReadingOverview,
    ReadingStreaks,
} from '../../../shared/contracts';

export type StatisticsScope = ScopeValue;

export type { ReadingOverview, ReadingStreaks, HeatmapConfig };

export interface DailyActivityEntry {
    date: string;
    reading_time_sec: number;
    pages_read: number;
}

export interface StatisticsWeekResponse {
    week_key: string;
    start_date: string;
    end_date: string;
    reading_time_sec: number;
    pages_read: number;
    longest_session_duration_sec: number | null;
    average_session_duration_sec: number | null;
    daily_activity: DailyActivityEntry[];
}

export interface StatisticsIndexWeek {
    week_key: string;
    start_date: string;
    end_date: string;
}

export interface StatisticsIndexResponse {
    available_years: number[];
    available_weeks: StatisticsIndexWeek[];
    overview: ReadingOverview;
    streaks: ReadingStreaks;
    heatmap_config: HeatmapConfig;
}

export interface StatisticsYearResponse {
    year: number;
    completions: number;
    daily_activity: DailyActivityEntry[];
    heatmap_config: HeatmapConfig;
}

function mergeDailyActivity(
    points: MetricPoint[],
): DailyActivityEntry[] {
    return points.map((p) => ({
        date: p.key,
        reading_time_sec: (p.reading_time_sec as number) ?? 0,
        pages_read: (p.pages_read as number) ?? 0,
    }));
}

export async function loadStatisticsIndex(
    scope: StatisticsScope,
): Promise<StatisticsIndexResponse> {
    const [summary, weekPeriods, yearPeriods] = await Promise.all([
        api.getReadingSummary(scope),
        api.getAvailablePeriods('reading_data', 'week', scope),
        api.getAvailablePeriods('reading_data', 'year', scope),
    ]);

    return {
        available_years: yearPeriods.periods.map((p) => Number(p.key)),
        available_weeks: weekPeriods.periods.map((p) => ({
            week_key: p.key,
            start_date: p.start_date,
            end_date: p.end_date,
        })),
        overview: summary.overview,
        streaks: summary.streaks,
        heatmap_config: summary.heatmap_config,
    };
}

export async function loadStatisticsWeek(
    scope: StatisticsScope,
    weekKey: string,
    startDate: string,
    endDate: string,
): Promise<StatisticsWeekResponse> {
    const [summary, metrics] = await Promise.all([
        api.getReadingSummary(scope, startDate, endDate),
        api.getReadingMetrics(
            scope,
            'reading_time_sec,pages_read',
            'day',
            startDate,
            endDate,
        ),
    ]);

    return {
        week_key: weekKey,
        start_date: startDate,
        end_date: endDate,
        reading_time_sec: summary.overview.reading_time_sec,
        pages_read: summary.overview.pages_read,
        longest_session_duration_sec:
            summary.overview.longest_session_duration_sec ?? null,
        average_session_duration_sec:
            summary.overview.average_session_duration_sec ?? null,
        daily_activity: mergeDailyActivity(metrics.points),
    };
}

export async function loadStatisticsYear(
    scope: StatisticsScope,
    year: number,
): Promise<StatisticsYearResponse> {
    const from = `${year}-01-01`;
    const to = `${year}-12-31`;

    const [summary, metrics] = await Promise.all([
        api.getReadingSummary(scope, from, to),
        api.getReadingMetrics(
            scope,
            'reading_time_sec,pages_read',
            'day',
            from,
            to,
        ),
    ]);

    return {
        year,
        completions: summary.overview.completions,
        daily_activity: mergeDailyActivity(metrics.points),
        heatmap_config: summary.heatmap_config,
    };
}
