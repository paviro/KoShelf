import {
    keepPreviousData,
    useQuery,
    type QueryClient,
} from '@tanstack/react-query';

import {
    loadRecapIndex,
    loadRecapYear,
    type RecapScope,
} from '../api/recap-data';

export function recapIndexQueryKey(scope: RecapScope) {
    return ['recap-index', scope] as const;
}

export function recapYearQueryKey(scope: RecapScope, year: number | null) {
    return ['recap-year', scope, year] as const;
}

function recapIndexQueryOptions(scope: RecapScope) {
    return {
        queryKey: recapIndexQueryKey(scope),
        queryFn: () => loadRecapIndex(scope),
    };
}

export function prefetchRecapIndexQuery(
    queryClient: QueryClient,
    scope: RecapScope,
): Promise<void> {
    return queryClient.prefetchQuery(recapIndexQueryOptions(scope));
}

export function useRecapIndexQuery(scope: RecapScope) {
    return useQuery({
        queryKey: recapIndexQueryKey(scope),
        queryFn: () => loadRecapIndex(scope),
        placeholderData: keepPreviousData,
    });
}

export function useRecapYearQuery(scope: RecapScope, year: number | null) {
    return useQuery({
        queryKey: recapYearQueryKey(scope, year),
        queryFn: () => loadRecapYear(scope, year ?? 0),
        enabled: year !== null,
        placeholderData: keepPreviousData,
    });
}
