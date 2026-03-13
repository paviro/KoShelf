import { api } from '../../../shared/api';
import type {
    CalendarItemRef,
    CalendarScopeStats,
    ReadingCalendarEvent,
} from '../../../shared/contracts';

export type CalendarContentType = 'book' | 'comic';

export type CalendarEventResponse = ReadingCalendarEvent;

export type CalendarItemResponse = CalendarItemRef;

export type CalendarMonthlyStats = CalendarScopeStats;

export interface CalendarScopedMonthlyStats {
    all: CalendarMonthlyStats;
    books: CalendarMonthlyStats;
    comics: CalendarMonthlyStats;
}

export interface CalendarMonthResponse {
    events: CalendarEventResponse[];
    items: Record<string, CalendarItemResponse>;
    stats: CalendarScopedMonthlyStats;
}

export interface CalendarMonthsResponse {
    months: string[];
}

export async function loadCalendarMonths(): Promise<CalendarMonthsResponse> {
    const data = await api.reading.availablePeriods(
        'reading_data',
        'month',
        'all',
    );
    return {
        months: data.periods.map((p) => p.key),
    };
}

export async function loadCalendarMonth(
    monthKey: string,
): Promise<CalendarMonthResponse> {
    const data = await api.reading.calendar(monthKey, 'all');

    return {
        events: data.events,
        items: data.items,
        stats: {
            all: data.stats_by_scope.all,
            books: data.stats_by_scope.books,
            comics: data.stats_by_scope.comics,
        },
    };
}
