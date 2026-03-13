import {
    keepPreviousData,
    useQuery,
    type QueryClient,
} from '@tanstack/react-query';

import {
    loadStatisticsIndex,
    loadStatisticsWeek,
    loadStatisticsYear,
    type StatisticsIndexWeek,
    type StatisticsScope,
} from '../api/statistics-data';

export function statisticsIndexQueryKey(scope: StatisticsScope) {
    return ['statistics-index', scope] as const;
}

export function statisticsWeekQueryKey(
    scope: StatisticsScope,
    weekKey: string | null,
) {
    return ['statistics-week', scope, weekKey] as const;
}

export function statisticsYearQueryKey(
    scope: StatisticsScope,
    year: number | null,
) {
    return ['statistics-year', scope, year] as const;
}

function statisticsIndexQueryOptions(scope: StatisticsScope) {
    return {
        queryKey: statisticsIndexQueryKey(scope),
        queryFn: () => loadStatisticsIndex(scope),
    };
}

export function prefetchStatisticsIndexQuery(
    queryClient: QueryClient,
    scope: StatisticsScope,
): Promise<void> {
    return queryClient.prefetchQuery(statisticsIndexQueryOptions(scope));
}

export function useStatisticsIndexQuery(scope: StatisticsScope) {
    return useQuery({
        queryKey: statisticsIndexQueryKey(scope),
        queryFn: () => loadStatisticsIndex(scope),
        placeholderData: keepPreviousData,
    });
}

export function useStatisticsWeekQuery(
    scope: StatisticsScope,
    week: StatisticsIndexWeek | null,
) {
    return useQuery({
        queryKey: statisticsWeekQueryKey(scope, week?.week_key ?? null),
        queryFn: () =>
            loadStatisticsWeek(
                scope,
                week!.week_key,
                week!.start_date,
                week!.end_date,
            ),
        enabled: week !== null,
        placeholderData: keepPreviousData,
    });
}

export function useStatisticsYearQuery(
    scope: StatisticsScope,
    year: number | null,
) {
    return useQuery({
        queryKey: statisticsYearQueryKey(scope, year),
        queryFn: () => loadStatisticsYear(scope, year ?? 0),
        enabled: Boolean(year),
        placeholderData: keepPreviousData,
    });
}
