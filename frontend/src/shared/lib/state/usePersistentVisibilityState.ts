import { useEffect, useState } from 'react';

import { StorageManager } from '../../storage-manager';

type StorageKey = (typeof StorageManager.KEYS)[keyof typeof StorageManager.KEYS];

export type PersistentVisibilityState<SectionKey extends string> = Record<SectionKey, boolean>;

type UsePersistentVisibilityStateOptions<SectionKey extends string> = {
    storageKey: StorageKey;
    sectionKeys: readonly SectionKey[];
    defaults: PersistentVisibilityState<SectionKey>;
};

function readStateFromStorage<SectionKey extends string>(
    storageKey: StorageKey,
    sectionKeys: readonly SectionKey[],
    defaults: PersistentVisibilityState<SectionKey>,
): PersistentVisibilityState<SectionKey> {
    const persisted = StorageManager.get<Record<string, boolean>>(storageKey, {});
    const nextState = { ...defaults };

    sectionKeys.forEach((sectionKey) => {
        const persistedValue = persisted?.[sectionKey];
        if (typeof persistedValue === 'boolean') {
            nextState[sectionKey] = persistedValue;
        }
    });

    return nextState;
}

export function usePersistentVisibilityState<SectionKey extends string>({
    storageKey,
    sectionKeys,
    defaults,
}: UsePersistentVisibilityStateOptions<SectionKey>) {
    const [state, setState] = useState<PersistentVisibilityState<SectionKey>>(() =>
        readStateFromStorage(storageKey, sectionKeys, defaults),
    );

    useEffect(() => {
        setState(readStateFromStorage(storageKey, sectionKeys, defaults));
    }, [defaults, sectionKeys, storageKey]);

    const setVisibility = (sectionKey: SectionKey, visible: boolean): void => {
        setState((current) => {
            const next = { ...current, [sectionKey]: visible };
            StorageManager.set(storageKey, next);
            return next;
        });
    };

    const toggle = (sectionKey: SectionKey): void => {
        setState((current) => {
            const next = { ...current, [sectionKey]: !current[sectionKey] };
            StorageManager.set(storageKey, next);
            return next;
        });
    };

    return {
        state,
        setVisibility,
        toggle,
    };
}
