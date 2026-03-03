import type { ScopeValue } from '../../../shared/api';
import type {
    CalendarEventResponse,
    CalendarItemResponse,
    CalendarMonthResponse,
    CalendarMonthlyStats,
} from '../api/calendar-data';

export const CALENDAR_FILTER_STORAGE_KEY = 'koshelf_calendar_filter';

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

    if (!Number.isFinite(year) || !Number.isFinite(month) || month < 1 || month > 12) {
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
    return now.getFullYear() === date.getFullYear() && now.getMonth() === date.getMonth();
}

export function aggregateCalendarData(months: CalendarMonthResponse[]): AggregatedCalendarData {
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
        return scope === 'books';
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

export function loadInitialScope(): ScopeValue {
    try {
        const raw = localStorage.getItem(CALENDAR_FILTER_STORAGE_KEY);
        if (raw === 'all') {
            return 'all';
        }
        if (raw === 'books' || raw === 'book') {
            return 'books';
        }
        if (raw === 'comics' || raw === 'comic') {
            return 'comics';
        }
    } catch {
        // Ignore storage read failures.
    }

    return 'all';
}

export function persistScope(scope: ScopeValue): void {
    try {
        localStorage.setItem(CALENDAR_FILTER_STORAGE_KEY, scope);
    } catch {
        // Ignore storage write failures.
    }
}
