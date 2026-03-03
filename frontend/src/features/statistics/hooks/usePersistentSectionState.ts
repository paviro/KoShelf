import { useMemo } from 'react';

import type { StatisticsScope } from '../api/statistics-data';
import { StorageManager } from '../../../shared/storage-manager';
import { usePersistentVisibilityState } from '../../../shared/lib/state/usePersistentVisibilityState';
import { SECTION_NAMES, defaultSectionState, type SectionName } from '../model/statistics-model';

function statsSectionStorageKey(
    scope: StatisticsScope,
):
    | typeof StorageManager.KEYS.STATS_ALL_SECTIONS
    | typeof StorageManager.KEYS.STATS_BOOKS_SECTIONS
    | typeof StorageManager.KEYS.STATS_COMICS_SECTIONS {
    if (scope === 'books') return StorageManager.KEYS.STATS_BOOKS_SECTIONS;
    if (scope === 'comics') return StorageManager.KEYS.STATS_COMICS_SECTIONS;
    return StorageManager.KEYS.STATS_ALL_SECTIONS;
}

export function usePersistentSectionState(scope: StatisticsScope) {
    const storageKey = useMemo(() => statsSectionStorageKey(scope), [scope]);
    const defaults = useMemo(() => defaultSectionState(), []);

    const { state, toggle } = usePersistentVisibilityState({
        storageKey,
        sectionKeys: SECTION_NAMES,
        defaults,
    });

    return {
        state,
        toggle: (sectionName: SectionName) => toggle(sectionName),
    };
}
