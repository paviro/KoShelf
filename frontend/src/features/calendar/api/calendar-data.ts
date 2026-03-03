import { api } from '../../../shared/api';

export type CalendarContentType = 'book' | 'comic';

export interface CalendarMonthsResponse {
    months: string[];
}

export interface CalendarEventResponse {
    item_id: string;
    start: string;
    end?: string;
    total_read_time: number;
    total_pages_read: number;
}

export interface CalendarItemResponse {
    title: string;
    authors: string[];
    content_type: CalendarContentType;
    color: string;
    item_path?: string;
    item_cover?: string;
}

export interface CalendarMonthlyStats {
    books_read: number;
    pages_read: number;
    time_read: number;
    days_read_pct: number;
}

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

export function createEmptyMonthlyStats(): CalendarMonthlyStats {
    return {
        books_read: 0,
        pages_read: 0,
        time_read: 0,
        days_read_pct: 0,
    };
}

export function createEmptyCalendarMonthResponse(): CalendarMonthResponse {
    const emptyStats = createEmptyMonthlyStats();

    return {
        events: [],
        items: {},
        stats: {
            all: { ...emptyStats },
            books: { ...emptyStats },
            comics: { ...emptyStats },
        },
    };
}

function isNotFoundError(error: unknown): boolean {
    return error instanceof Error && /\(404\)$/.test(error.message);
}

export async function loadCalendarMonths(): Promise<CalendarMonthsResponse> {
    return api.calendar.months.list<CalendarMonthsResponse>();
}

export async function loadCalendarMonth(monthKey: string): Promise<CalendarMonthResponse> {
    try {
        return await api.calendar.months.get<CalendarMonthResponse>(monthKey);
    } catch (error) {
        if (isNotFoundError(error)) {
            return createEmptyCalendarMonthResponse();
        }

        throw error;
    }
}
