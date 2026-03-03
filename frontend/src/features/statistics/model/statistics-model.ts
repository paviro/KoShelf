import type {
    DailyActivityEntry,
    StatisticsIndexWeek,
    StatisticsScope,
} from '../../../shared/statistics-data-loader';
import { DateFormatter } from '../../../shared/statistics-formatters';
import { translation } from '../../../shared/i18n';
import { monthKeyAt } from '../../../shared/statistics-months';

export const SECTION_NAMES = [
    'overall-stats',
    'reading-streak',
    'yearly-stats',
    'weekly-stats',
] as const;

export type SectionName = (typeof SECTION_NAMES)[number];

export type SectionVisibilityState = Record<SectionName, boolean>;

export type MonthlyReadStats = {
    read_time: number;
    pages_read: number;
    active_days: number;
};

export type YearlySummaryStats = {
    read_time: number;
    completed_count: number;
    active_days: number;
};

export function normalizeScope(scope: string | undefined): StatisticsScope {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}

export function defaultSectionState(): SectionVisibilityState {
    return {
        'overall-stats': true,
        'reading-streak': true,
        'yearly-stats': true,
        'weekly-stats': true,
    };
}

export function formatReadTimeWithWeeks(seconds: number): string {
    const totalHours = Math.floor(seconds / 3600);
    const totalDays = Math.floor(totalHours / 24);
    const weeks = Math.floor(totalDays / 7);
    const days = totalDays % 7;
    const hours = totalHours % 24;
    const minutes = Math.floor((seconds % 3600) / 60);

    if (weeks > 0) {
        return `${weeks}${translation.get('units.w')} ${days}${translation.get('units.d')} ${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
    }
    if (days > 0) {
        return `${days}${translation.get('units.d')} ${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
    }
    if (hours > 0) {
        return `${hours}${translation.get('units.h')} ${minutes}${translation.get('units.m')}`;
    }
    return `${minutes}${translation.get('units.m')}`;
}

export function formatSessionDuration(seconds: number | null): string {
    if (seconds === null) {
        return '--';
    }

    const minutes = Math.floor(seconds / 60);
    if (minutes >= 60) {
        const hours = Math.floor(minutes / 60);
        const remaining = minutes % 60;
        return `${hours}${translation.get('units.h')} ${remaining}${translation.get('units.m')}`;
    }

    return `${minutes}${translation.get('units.m')}`;
}

function tryFormatSingleDayRange(dateStr: string): string | null {
    const day = Number.parseInt(dateStr.slice(8, 10), 10);
    const month = Number.parseInt(dateStr.slice(5, 7), 10) - 1;
    const year = Number.parseInt(dateStr.slice(0, 4), 10);

    if (Number.isNaN(day) || Number.isNaN(month) || Number.isNaN(year) || month < 0 || month > 11) {
        return null;
    }

    const currentYear = new Date().getFullYear();
    const includeYear = year !== currentYear;
    return includeYear
        ? `${day} ${translation.get(monthKeyAt(month))} ${year}`
        : `${day} ${translation.get(monthKeyAt(month))}`;
}

function formatSingleStreakDate(dateStr: string): string {
    return tryFormatSingleDayRange(dateStr) ?? dateStr;
}

export function formatStreakDateRange(startDate: string | null, endDate: string | null): string {
    if (!startDate && !endDate) {
        return '';
    }

    if (startDate && !endDate) {
        return `${formatSingleStreakDate(startDate)} - now`;
    }

    if (!startDate || !endDate) {
        return '';
    }

    if (startDate === endDate) {
        return (
            tryFormatSingleDayRange(startDate) ??
            DateFormatter.formatDateRange(startDate, endDate, 'long')
        );
    }

    const formattedStart = formatSingleStreakDate(startDate);
    const formattedEnd = formatSingleStreakDate(endDate);
    return `${formattedStart} - ${formattedEnd}`;
}

export function isCurrentStreakActive(endDate: string | null): boolean {
    if (!endDate) {
        return false;
    }

    const today = new Date();
    const todayIso = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(
        today.getDate(),
    ).padStart(2, '0')}`;
    return endDate === todayIso;
}

export function aggregateMonthlyStats(dailyActivity: DailyActivityEntry[]): MonthlyReadStats[] {
    const monthlyStats = Array.from({ length: 12 }, () => ({
        read_time: 0,
        pages_read: 0,
        active_days: 0,
    }));

    dailyActivity.forEach((entry) => {
        const month = Number.parseInt(entry.date.slice(5, 7), 10) - 1;
        if (Number.isNaN(month) || month < 0 || month > 11) {
            return;
        }

        monthlyStats[month].read_time += entry.read_time;
        monthlyStats[month].pages_read += entry.pages_read;

        if (entry.read_time > 0 || entry.pages_read > 0) {
            monthlyStats[month].active_days += 1;
        }
    });

    return monthlyStats;
}

export function summarizeYearlyStats(
    monthlyStats: MonthlyReadStats[],
    completedCount: number,
): YearlySummaryStats {
    const summary: YearlySummaryStats = {
        read_time: 0,
        completed_count: completedCount,
        active_days: 0,
    };

    monthlyStats.forEach((month) => {
        summary.read_time += month.read_time;
        summary.active_days += month.active_days;
    });

    return summary;
}

export function getWeekYearOrder(weeks: StatisticsIndexWeek[]): string[] {
    const years: string[] = [];
    for (const week of weeks) {
        const year = week.start_date.substring(0, 4);
        if (!years.includes(year)) {
            years.push(year);
        }
    }
    return years;
}
