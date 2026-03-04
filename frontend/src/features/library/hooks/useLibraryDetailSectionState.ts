import { useMemo } from 'react';

import { usePersistentVisibilityState } from '../../../shared/lib/state/usePersistentVisibilityState';
import {
    LIBRARY_DETAIL_SECTION_KEYS,
    defaultLibraryDetailSectionState,
    libraryDetailSectionStorageKey,
    type LibraryDetailSectionKey,
} from '../model/library-detail-model';
import type { LibraryCollection } from '../model/library-model';

export function useLibraryDetailSectionState(collection: LibraryCollection) {
    const storageKey = useMemo(() => libraryDetailSectionStorageKey(collection), [collection]);
    const defaults = useMemo(() => defaultLibraryDetailSectionState(), []);

    const { state, toggle } = usePersistentVisibilityState({
        storageKey,
        sectionKeys: LIBRARY_DETAIL_SECTION_KEYS,
        defaults,
    });

    return {
        state,
        toggle: (sectionKey: LibraryDetailSectionKey) => toggle(sectionKey),
    };
}
