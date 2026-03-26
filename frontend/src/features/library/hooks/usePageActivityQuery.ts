import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type { PageActivityData } from '../../../shared/contracts';

export function pageActivityQueryKey(
    id: string | undefined,
    completion?: string,
) {
    return ['page-activity', id, completion ?? 'all'] as const;
}

export function usePageActivityQuery(
    id: string | undefined,
    enabled: boolean,
    completion?: string,
) {
    return useQuery<PageActivityData>({
        queryKey: pageActivityQueryKey(id, completion),
        queryFn: () => api.getItemPageActivity(id!, completion),
        enabled: Boolean(id) && enabled,
        staleTime: 5 * 60 * 1000,
        placeholderData: keepPreviousData,
    });
}
