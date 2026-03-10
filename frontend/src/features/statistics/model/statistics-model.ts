import type {
    DailyActivityEntry,
    StatisticsIndexWeek,
    StatisticsScope,
} from '../api/statistics-data';
import { translation } from '../../../shared/i18n';
import {
    formatPlainDate,
    formatPlainDateRange,
} from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { type UnitValuePart } from '../../../shared/lib/intl/unit-value';
import {
    patchRouteState,
    readRouteState,
} from '../../../shared/lib/state/route-state-storage';

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

export type StatisticsViewState = {
    scope: StatisticsScope;
    selectedWeekKey: string | null;
    selectedHeatmapYear: number | null;
    selectedYearlyYear: number | null;
};

function normalizeSelectedYear(year: unknown): number | null {
    if (typeof year !== 'number' || !Number.isFinite(year)) {
        return null;
    }

    const rounded = Math.floor(year);
    if (rounded < 1900 || rounded > 9999) {
        return null;
    }

    return rounded;
}

function normalizeSelectedWeekKey(weekKey: unknown): string | null {
    if (typeof weekKey !== 'string') {
        return null;
    }

    const trimmed = weekKey.trim();
    return trimmed.length > 0 ? trimmed : null;
}

export function normalizeScope(scope: unknown): StatisticsScope {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}

export function readStoredStatisticsViewState(): StatisticsViewState {
    const persisted = readRouteState('statistics', 'session');
    return {
        scope: normalizeScope(persisted.scope),
        selectedWeekKey: normalizeSelectedWeekKey(persisted.selectedWeekKey),
        selectedHeatmapYear: normalizeSelectedYear(
            persisted.selectedHeatmapYear,
        ),
        selectedYearlyYear: normalizeSelectedYear(persisted.selectedYearlyYear),
    };
}

export function persistStatisticsViewState(state: StatisticsViewState): void {
    patchRouteState('statistics', 'session', {
        scope: normalizeScope(state.scope),
        selectedWeekKey: normalizeSelectedWeekKey(state.selectedWeekKey),
        selectedHeatmapYear: normalizeSelectedYear(state.selectedHeatmapYear),
        selectedYearlyYear: normalizeSelectedYear(state.selectedYearlyYear),
    });
}

export function defaultSectionState(): SectionVisibilityState {
    return {
        'overall-stats': true,
        'reading-streak': true,
        'yearly-stats': true,
        'weekly-stats': true,
    };
}

type UnitKey = 'units.w' | 'units.d' | 'units.h' | 'units.m';

function formatUnitPart(value: number, unit: UnitKey): UnitValuePart {
    return {
        amount: formatNumber(value),
        unit: translation.get(unit),
    };
}

export function formatReadTimeWithWeeksParts(seconds: number): UnitValuePart[] {
    if (!Number.isFinite(seconds)) {
        return [{ amount: '--' }];
    }

    const normalizedSeconds = Math.max(0, Math.floor(seconds));
    const totalHours = Math.floor(normalizedSeconds / 3600);
    const totalDays = Math.floor(totalHours / 24);
    const weeks = Math.floor(totalDays / 7);
    const days = totalDays % 7;
    const hours = totalHours % 24;
    const minutes = Math.floor((normalizedSeconds % 3600) / 60);

    if (weeks > 0) {
        return [
            formatUnitPart(weeks, 'units.w'),
            formatUnitPart(days, 'units.d'),
            formatUnitPart(hours, 'units.h'),
            formatUnitPart(minutes, 'units.m'),
        ];
    }
    if (days > 0) {
        return [
            formatUnitPart(days, 'units.d'),
            formatUnitPart(hours, 'units.h'),
            formatUnitPart(minutes, 'units.m'),
        ];
    }
    if (hours > 0) {
        return [
            formatUnitPart(hours, 'units.h'),
            formatUnitPart(minutes, 'units.m'),
        ];
    }
    return [formatUnitPart(minutes, 'units.m')];
}

export function formatSessionDurationParts(
    seconds: number | null,
): UnitValuePart[] {
    if (seconds === null || !Number.isFinite(seconds)) {
        return [{ amount: '--' }];
    }

    const minutes = Math.max(0, Math.floor(seconds / 60));
    if (minutes >= 60) {
        const hours = Math.floor(minutes / 60);
        const remaining = minutes % 60;
        return [
            formatUnitPart(hours, 'units.h'),
            formatUnitPart(remaining, 'units.m'),
        ];
    }

    return [formatUnitPart(minutes, 'units.m')];
}

function formatSingleStreakDate(dateStr: string): string {
    return formatPlainDate(dateStr, {
        monthStyle: 'long',
        yearDisplay: 'auto',
        fallback: dateStr,
    });
}

export function formatStreakDateRange(
    startDate: string | null,
    endDate: string | null,
): string {
    if (!startDate && !endDate) {
        return '';
    }

    if (startDate && !endDate) {
        return `${formatSingleStreakDate(startDate)} – ${translation.get('today')}`;
    }

    if (!startDate || !endDate) {
        return '';
    }

    if (startDate === endDate) {
        return formatSingleStreakDate(startDate);
    }

    return formatPlainDateRange(startDate, endDate, {
        monthStyle: 'long',
        yearDisplay: 'auto',
    });
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

export function aggregateMonthlyStats(
    dailyActivity: DailyActivityEntry[],
): MonthlyReadStats[] {
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
