import { useQuery } from '@tanstack/react-query';

import {
    loadStatisticsIndex,
    loadStatisticsWeek,
    loadStatisticsYear,
    type StatisticsScope,
} from '../api/statistics-data';

export function useStatisticsIndexQuery(scope: StatisticsScope) {
    return useQuery({
        queryKey: ['statistics-index', scope],
        queryFn: () => loadStatisticsIndex(scope),
    });
}

export function useStatisticsWeekQuery(scope: StatisticsScope, weekKey: string | null) {
    return useQuery({
        queryKey: ['statistics-week', scope, weekKey],
        queryFn: () => loadStatisticsWeek(scope, weekKey ?? ''),
        enabled: Boolean(weekKey),
    });
}

export function useStatisticsYearQuery(scope: StatisticsScope, year: number | null) {
    return useQuery({
        queryKey: ['statistics-year', scope, year],
        queryFn: () => loadStatisticsYear(scope, year ?? 0),
        enabled: Boolean(year),
    });
}
