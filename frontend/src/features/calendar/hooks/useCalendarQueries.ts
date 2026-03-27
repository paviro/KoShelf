import {
    keepPreviousData,
    useQuery,
    type QueryClient,
} from '@tanstack/react-query';

import { loadCalendarMonth, loadCalendarMonths } from '../api/calendar-data';

function calendarMonthsQueryKey() {
    return ['calendar-months'] as const;
}

function calendarMonthQueryKey(monthKey: string) {
    return ['calendar-month', monthKey] as const;
}

function calendarMonthQueryOptions(monthKey: string) {
    return {
        queryKey: calendarMonthQueryKey(monthKey),
        queryFn: () => loadCalendarMonth(monthKey),
    };
}

function prefetchCalendarMonthQuery(
    queryClient: QueryClient,
    monthKey: string,
): Promise<void> {
    return queryClient.prefetchQuery(calendarMonthQueryOptions(monthKey));
}

export async function prefetchCalendarMonthsIfAvailable(
    queryClient: QueryClient,
    monthKeys: string[],
): Promise<void> {
    const { months } = await queryClient.fetchQuery({
        queryKey: calendarMonthsQueryKey(),
        queryFn: loadCalendarMonths,
    });
    const available = new Set(months);
    await Promise.all(
        monthKeys
            .filter((key) => available.has(key))
            .map((key) => prefetchCalendarMonthQuery(queryClient, key)),
    );
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
