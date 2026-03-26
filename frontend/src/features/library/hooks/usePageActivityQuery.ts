import { useQuery } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type { PageActivityData } from '../../../shared/contracts';

export function pageActivityQueryKey(id: string | undefined) {
    return ['page-activity', id] as const;
}

export function usePageActivityQuery(id: string | undefined, enabled: boolean) {
    return useQuery<PageActivityData>({
        queryKey: pageActivityQueryKey(id),
        queryFn: () => api.getItemPageActivity(id!),
        enabled: Boolean(id) && enabled,
        staleTime: 5 * 60 * 1000,
    });
}
