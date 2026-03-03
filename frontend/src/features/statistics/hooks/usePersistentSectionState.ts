import { useEffect, useMemo, useState } from 'react';

import type { StatisticsScope } from '../api/statistics-data';
import { StorageManager } from '../../../shared/storage-manager';
import {
    SECTION_NAMES,
    defaultSectionState,
    type SectionName,
    type SectionVisibilityState,
} from '../model/statistics-model';

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

function buildStateFromStorage(
    storageKey: ReturnType<typeof statsSectionStorageKey>,
): SectionVisibilityState {
    const persisted = StorageManager.get<Record<string, boolean>>(storageKey, {});
    const next = defaultSectionState();

    for (const sectionName of SECTION_NAMES) {
        if (typeof persisted?.[sectionName] === 'boolean') {
            next[sectionName] = persisted[sectionName];
        }
    }

    return next;
}

export function usePersistentSectionState(scope: StatisticsScope) {
    const storageKey = useMemo(() => statsSectionStorageKey(scope), [scope]);
    const [state, setState] = useState<SectionVisibilityState>(() =>
        buildStateFromStorage(storageKey),
    );

    useEffect(() => {
        setState(buildStateFromStorage(storageKey));
    }, [storageKey]);

    const toggle = (sectionName: SectionName): void => {
        setState((current) => {
            const next = { ...current, [sectionName]: !current[sectionName] };
            StorageManager.set(storageKey, next);
            return next;
        });
    };

    return { state, toggle };
}
