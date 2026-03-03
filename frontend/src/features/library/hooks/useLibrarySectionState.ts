import { useMemo } from 'react';

import { usePersistentVisibilityState } from '../../../shared/lib/state/usePersistentVisibilityState';
import {
    LIBRARY_SECTION_KEYS,
    defaultLibrarySectionState,
    librarySectionStorageKey,
    type LibraryCollection,
    type LibrarySectionKey,
} from '../model/library-model';

export function useLibrarySectionState(collection: LibraryCollection) {
    const storageKey = useMemo(() => librarySectionStorageKey(collection), [collection]);
    const defaults = useMemo(() => defaultLibrarySectionState(), []);

    const { state, toggle } = usePersistentVisibilityState({
        storageKey,
        sectionKeys: LIBRARY_SECTION_KEYS,
        defaults,
    });

    return {
        state,
        toggle: (sectionName: LibrarySectionKey) => toggle(sectionName),
    };
}
