import { useQuery } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type { LibraryListResponse } from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';

async function fetchLibraryList(collection: LibraryCollection): Promise<LibraryListResponse> {
    if (collection === 'comics') {
        return api.comics.list<LibraryListResponse>();
    }

    return api.books.list<LibraryListResponse>();
}

export function useLibraryListQuery(collection: LibraryCollection) {
    return useQuery({
        queryKey: ['library-list', collection],
        queryFn: () => fetchLibraryList(collection),
    });
}
