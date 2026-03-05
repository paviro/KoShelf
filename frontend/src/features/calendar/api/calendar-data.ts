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
    item_id?: string;
    item_cover?: string;
}

export interface CalendarMonthlyStats {
    items_read: number;
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

type ActivityMonthPayload = {
    events: CalendarEventResponse[];
    items: Record<string, CalendarItemResponse>;
    stats: CalendarMonthlyStats;
};

export async function loadCalendarMonths(): Promise<CalendarMonthsResponse> {
    return api.activity.months.list<CalendarMonthsResponse>('all');
}

export async function loadCalendarMonth(
    monthKey: string,
): Promise<CalendarMonthResponse> {
    const [allPayload, booksPayload, comicsPayload] = await Promise.all([
        api.activity.months.get<ActivityMonthPayload>(monthKey, 'all'),
        api.activity.months.get<ActivityMonthPayload>(monthKey, 'books'),
        api.activity.months.get<ActivityMonthPayload>(monthKey, 'comics'),
    ]);

    return {
        events: allPayload.events,
        items: allPayload.items,
        stats: {
            all: allPayload.stats,
            books: booksPayload.stats,
            comics: comicsPayload.stats,
        },
    };
}
