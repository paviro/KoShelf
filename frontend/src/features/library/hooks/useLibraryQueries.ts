import { useQuery } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type { LibraryDetailResponse, LibraryListResponse } from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';

async function fetchLibraryList(collection: LibraryCollection): Promise<LibraryListResponse> {
    if (collection === 'comics') {
        return api.comics.list<LibraryListResponse>();
    }

    return api.books.list<LibraryListResponse>();
}

async function fetchLibraryDetail(
    collection: LibraryCollection,
    id: string,
): Promise<LibraryDetailResponse> {
    if (collection === 'comics') {
        return api.comics.get<LibraryDetailResponse>(id);
    }

    return api.books.get<LibraryDetailResponse>(id);
}

export function useLibraryListQuery(collection: LibraryCollection) {
    return useQuery({
        queryKey: ['library-list', collection],
        queryFn: () => fetchLibraryList(collection),
    });
}

export function useLibraryDetailQuery(collection: LibraryCollection, id: string | undefined) {
    return useQuery({
        queryKey: ['library-detail', collection, id],
        queryFn: () => fetchLibraryDetail(collection, id ?? ''),
        enabled: Boolean(id),
    });
}
