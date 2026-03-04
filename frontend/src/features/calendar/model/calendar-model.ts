import type { ScopeValue } from '../../../shared/api';
import {
    patchRouteState,
    readRouteState,
} from '../../../shared/lib/state/route-state-storage';
import type {
    CalendarEventResponse,
    CalendarItemResponse,
    CalendarMonthResponse,
    CalendarMonthlyStats,
} from '../api/calendar-data';

type AggregatedCalendarData = {
    events: CalendarEventResponse[];
    items: Record<string, CalendarItemResponse>;
};

function safeDate(year: number, monthIndex: number, date: number): Date {
    return new Date(year, monthIndex, date, 12, 0, 0, 0);
}

export function monthKey(date: Date): string {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
}

export function parseMonthKey(targetMonthKey: string): Date {
    const [yearRaw, monthRaw] = targetMonthKey.split('-');
    const year = Number(yearRaw);
    const month = Number(monthRaw);

    if (
        !Number.isFinite(year) ||
        !Number.isFinite(month) ||
        month < 1 ||
        month > 12
    ) {
        return normalizeToMonthStart(new Date());
    }

    return safeDate(year, month - 1, 1);
}

export function normalizeToMonthStart(date: Date): Date {
    return safeDate(date.getFullYear(), date.getMonth(), 1);
}

export function shiftMonth(date: Date, offset: number): Date {
    return safeDate(date.getFullYear(), date.getMonth() + offset, 1);
}

export function shiftMonthKey(targetMonthKey: string, offset: number): string {
    return monthKey(shiftMonth(parseMonthKey(targetMonthKey), offset));
}

export function isCurrentMonth(date: Date): boolean {
    const now = new Date();
    return (
        now.getFullYear() === date.getFullYear() &&
        now.getMonth() === date.getMonth()
    );
}

export function aggregateCalendarData(
    months: CalendarMonthResponse[],
): AggregatedCalendarData {
    const events: CalendarEventResponse[] = [];
    const items: Record<string, CalendarItemResponse> = {};
    const seen = new Set<string>();

    for (const monthData of months) {
        Object.assign(items, monthData.items);

        for (const event of monthData.events) {
            const dedupeKey = `${event.item_id}|${event.start}|${event.end ?? ''}`;
            if (seen.has(dedupeKey)) {
                continue;
            }

            seen.add(dedupeKey);
            events.push(event);
        }
    }

    return { events, items };
}

export function eventMatchesScope(
    event: CalendarEventResponse,
    items: Record<string, CalendarItemResponse>,
    scope: ScopeValue,
): boolean {
    if (scope === 'all') {
        return true;
    }

    const item = items[event.item_id];
    if (!item) {
        return false;
    }

    if (scope === 'books') {
        return item.content_type === 'book';
    }

    return item.content_type === 'comic';
}

export function resolveMonthlyStats(
    monthData: CalendarMonthResponse | undefined,
    scope: ScopeValue,
): CalendarMonthlyStats {
    if (!monthData) {
        return {
            books_read: 0,
            pages_read: 0,
            time_read: 0,
            days_read_pct: 0,
        };
    }

    if (scope === 'books') {
        return monthData.stats.books;
    }

    if (scope === 'comics') {
        return monthData.stats.comics;
    }

    return monthData.stats.all;
}

export function formatDuration(totalSeconds: number): string {
    if (!Number.isFinite(totalSeconds) || totalSeconds <= 0) {
        return '0s';
    }

    if (totalSeconds < 60) {
        return `${Math.floor(totalSeconds)}s`;
    }

    if (totalSeconds < 3600) {
        const minutes = Math.floor(totalSeconds / 60);
        const seconds = Math.floor(totalSeconds % 60);
        return seconds > 0 ? `${minutes}m ${seconds}s` : `${minutes}m`;
    }

    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
}

export type CalendarViewState = {
    scope: ScopeValue;
    monthKey: string | null;
};

function normalizeCalendarScope(value: unknown): ScopeValue {
    if (value === 'books' || value === 'book') {
        return 'books';
    }

    if (value === 'comics' || value === 'comic') {
        return 'comics';
    }

    return 'all';
}

function normalizeCalendarMonthKey(value: unknown): string | null {
    if (typeof value !== 'string') {
        return null;
    }

    const trimmed = value.trim();
    if (!/^\d{4}-\d{2}$/.test(trimmed)) {
        return null;
    }

    return monthKey(parseMonthKey(trimmed));
}

export function loadInitialCalendarViewState(): CalendarViewState {
    const persisted = readRouteState('calendar', 'session');

    return {
        scope: normalizeCalendarScope(persisted.scope),
        monthKey: normalizeCalendarMonthKey(persisted.monthKey),
    };
}

export function persistCalendarViewState(state: CalendarViewState): void {
    patchRouteState('calendar', 'session', {
        scope: normalizeCalendarScope(state.scope),
        monthKey: normalizeCalendarMonthKey(state.monthKey),
    });
}
