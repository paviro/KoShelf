import {
    keepPreviousData,
    useQuery,
    type QueryClient,
} from '@tanstack/react-query';

import { loadCalendarMonth, loadCalendarMonths } from '../api/calendar-data';

export function calendarMonthsQueryKey() {
    return ['calendar-months'] as const;
}

export function calendarMonthQueryKey(monthKey: string) {
    return ['calendar-month', monthKey] as const;
}

function calendarMonthQueryOptions(monthKey: string) {
    return {
        queryKey: calendarMonthQueryKey(monthKey),
        queryFn: () => loadCalendarMonth(monthKey),
    };
}

export function prefetchCalendarMonthQuery(
    queryClient: QueryClient,
    monthKey: string,
): Promise<void> {
    return queryClient.prefetchQuery(calendarMonthQueryOptions(monthKey));
}

export function useCalendarMonthsQuery() {
    return useQuery({
        queryKey: calendarMonthsQueryKey(),
        queryFn: loadCalendarMonths,
    });
}

export function useCalendarMonthQuery(monthKey: string, enabled = true) {
    return useQuery({
        queryKey: calendarMonthQueryKey(monthKey),
        queryFn: () => loadCalendarMonth(monthKey),
        enabled,
        placeholderData: keepPreviousData,
    });
}
