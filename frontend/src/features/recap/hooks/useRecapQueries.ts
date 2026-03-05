import { keepPreviousData, useQuery } from '@tanstack/react-query';

import {
    loadRecapIndex,
    loadRecapYear,
    type RecapScope,
} from '../api/recap-data';

export function useRecapIndexQuery(scope: RecapScope) {
    return useQuery({
        queryKey: ['recap-index', scope],
        queryFn: () => loadRecapIndex(scope),
    });
}

export function useRecapYearQuery(scope: RecapScope, year: number | null) {
    return useQuery({
        queryKey: ['recap-year', scope, year],
        queryFn: () => loadRecapYear(scope, year ?? 0),
        enabled: year !== null,
        placeholderData: keepPreviousData,
    });
}
