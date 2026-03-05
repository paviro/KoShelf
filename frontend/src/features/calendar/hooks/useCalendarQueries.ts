import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { loadCalendarMonth, loadCalendarMonths } from '../api/calendar-data';

export function useCalendarMonthsQuery() {
    return useQuery({
        queryKey: ['calendar-months'],
        queryFn: loadCalendarMonths,
    });
}

export function useCalendarMonthQuery(monthKey: string, enabled = true) {
    return useQuery({
        queryKey: ['calendar-month', monthKey],
        queryFn: () => loadCalendarMonth(monthKey),
        enabled,
        placeholderData: keepPreviousData,
    });
}
