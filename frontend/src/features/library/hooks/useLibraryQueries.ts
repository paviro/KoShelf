import {
    keepPreviousData,
    useQuery,
    type QueryClient,
} from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type {
    LibraryDetailData,
    LibraryListData,
} from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';

async function fetchLibraryList(
    collection: LibraryCollection,
): Promise<LibraryListData> {
    return api.getItems(
        collection === 'comics' ? 'comics' : 'books',
    );
}

export function libraryListQueryKey(collection: LibraryCollection) {
    return ['library-list', collection] as const;
}

function libraryListQueryOptions(collection: LibraryCollection) {
    return {
        queryKey: libraryListQueryKey(collection),
        queryFn: () => fetchLibraryList(collection),
    };
}

export function prefetchLibraryListQuery(
    queryClient: QueryClient,
    collection: LibraryCollection,
): Promise<void> {
    return queryClient.prefetchQuery(libraryListQueryOptions(collection));
}

async function fetchLibraryDetail(
    collection: LibraryCollection,
    id: string,
): Promise<LibraryDetailData> {
    const detail = await api.getItem(id);
    if (collection === 'comics' && detail.item.content_type !== 'comic') {
        throw new Error(`Item ${id} is not a comic`);
    }
    if (collection === 'books' && detail.item.content_type !== 'book') {
        throw new Error(`Item ${id} is not a book`);
    }
    return detail;
}

export function libraryDetailQueryKey(
    collection: LibraryCollection,
    id: string | undefined,
) {
    return ['library-detail', collection, id] as const;
}

export function libraryDetailCacheKey(
    collection: LibraryCollection,
    id: string,
): string {
    return `${collection}:${id}`;
}

function libraryDetailQueryOptions(collection: LibraryCollection, id: string) {
    return {
        queryKey: libraryDetailQueryKey(collection, id),
        queryFn: () => fetchLibraryDetail(collection, id),
    };
}

export function prefetchLibraryDetailQuery(
    queryClient: QueryClient,
    collection: LibraryCollection,
    id: string,
): Promise<void> {
    return queryClient.prefetchQuery(libraryDetailQueryOptions(collection, id));
}

export function fetchLibraryDetailQuery(
    queryClient: QueryClient,
    collection: LibraryCollection,
    id: string,
): Promise<LibraryDetailData> {
    return queryClient.fetchQuery(libraryDetailQueryOptions(collection, id));
}

export function getCachedLibraryDetailQueryData(
    queryClient: QueryClient,
    collection: LibraryCollection,
    id: string,
): LibraryDetailData | undefined {
    return queryClient.getQueryData<LibraryDetailData>(
        libraryDetailQueryKey(collection, id),
    );
}

export function useLibraryListQuery(collection: LibraryCollection) {
    return useQuery({
        queryKey: libraryListQueryKey(collection),
        queryFn: () => fetchLibraryList(collection),
        placeholderData: keepPreviousData,
    });
}

export function useLibraryDetailQuery(
    collection: LibraryCollection,
    id: string | undefined,
) {
    return useQuery({
        queryKey: libraryDetailQueryKey(collection, id),
        queryFn: () => fetchLibraryDetail(collection, id ?? ''),
        enabled: Boolean(id),
        placeholderData: keepPreviousData,
    });
}
