import { keepPreviousData, useQuery } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type {
    LibraryDetailResponse,
    LibraryListResponse,
} from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';

async function fetchLibraryList(
    collection: LibraryCollection,
): Promise<LibraryListResponse> {
    return api.items.list<LibraryListResponse>(
        collection === 'comics' ? 'comics' : 'books',
    );
}

async function fetchLibraryDetail(
    collection: LibraryCollection,
    id: string,
): Promise<LibraryDetailResponse> {
    const detail = await api.items.get<LibraryDetailResponse>(id);
    if (collection === 'comics' && detail.item.content_type !== 'comic') {
        throw new Error(`Item ${id} is not a comic`);
    }
    if (collection === 'books' && detail.item.content_type !== 'book') {
        throw new Error(`Item ${id} is not a book`);
    }
    return detail;
}

export function useLibraryListQuery(collection: LibraryCollection) {
    return useQuery({
        queryKey: ['library-list', collection],
        queryFn: () => fetchLibraryList(collection),
        placeholderData: keepPreviousData,
    });
}

export function useLibraryDetailQuery(
    collection: LibraryCollection,
    id: string | undefined,
) {
    return useQuery({
        queryKey: ['library-detail', collection, id],
        queryFn: () => fetchLibraryDetail(collection, id ?? ''),
        enabled: Boolean(id),
        placeholderData: keepPreviousData,
    });
}
