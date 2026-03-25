import { useQuery } from '@tanstack/react-query';

import { api } from '../api';

export function useSiteQuery() {
    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.getSite(),
    });

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books &&
        siteQuery.data?.capabilities.has_comics,
    );

    return { siteQuery, showTypeFilter };
}
