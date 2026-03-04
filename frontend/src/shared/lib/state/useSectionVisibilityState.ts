import { useState } from 'react';

import type { RouteId } from '../../../app/routes/route-registry';
import { patchRouteState, readRouteState } from './route-state-storage';

export type SectionVisibilityState<SectionKey extends string> = Record<SectionKey, boolean>;

type SectionVisibilityOptions<SectionKey extends string> = {
    routeId: RouteId;
    sectionKeys: readonly SectionKey[];
    defaults: SectionVisibilityState<SectionKey>;
    fieldName?: string;
};

const DEFAULT_FIELD_NAME = 'sectionVisibility';

function normalizeSectionState<SectionKey extends string>(
    value: unknown,
    sectionKeys: readonly SectionKey[],
    defaults: SectionVisibilityState<SectionKey>,
): SectionVisibilityState<SectionKey> {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        return { ...defaults };
    }

    const parsed = value as Record<string, unknown>;
    const next = { ...defaults };
    sectionKeys.forEach((sectionKey) => {
        const persisted = parsed[sectionKey];
        if (typeof persisted === 'boolean') {
            next[sectionKey] = persisted;
        }
    });

    return next;
}

export function readSectionVisibilityState<SectionKey extends string>({
    routeId,
    sectionKeys,
    defaults,
    fieldName = DEFAULT_FIELD_NAME,
}: SectionVisibilityOptions<SectionKey>): SectionVisibilityState<SectionKey> {
    const persisted = readRouteState(routeId, 'local');
    return normalizeSectionState(persisted[fieldName], sectionKeys, defaults);
}

export function writeSectionVisibilityState<SectionKey extends string>(
    {
        routeId,
        sectionKeys,
        defaults,
        fieldName = DEFAULT_FIELD_NAME,
    }: SectionVisibilityOptions<SectionKey>,
    value: unknown,
): void {
    const normalized = normalizeSectionState(value, sectionKeys, defaults);
    patchRouteState(routeId, 'local', {
        [fieldName]: normalized,
    });
}

export function useSectionVisibilityState<SectionKey extends string>({
    routeId,
    sectionKeys,
    defaults,
    fieldName = DEFAULT_FIELD_NAME,
}: SectionVisibilityOptions<SectionKey>) {
    const [, setRevision] = useState(0);
    const state = readSectionVisibilityState({
        routeId,
        sectionKeys,
        defaults,
        fieldName,
    });

    const setVisibility = (sectionKey: SectionKey, visible: boolean): void => {
        const current = readSectionVisibilityState({
            routeId,
            sectionKeys,
            defaults,
            fieldName,
        });
        const next = { ...current, [sectionKey]: visible };
        writeSectionVisibilityState(
            {
                routeId,
                sectionKeys,
                defaults,
                fieldName,
            },
            next,
        );
        setRevision((value) => value + 1);
    };

    const toggle = (sectionKey: SectionKey): void => {
        const current = readSectionVisibilityState({
            routeId,
            sectionKeys,
            defaults,
            fieldName,
        });
        const next = { ...current, [sectionKey]: !current[sectionKey] };
        writeSectionVisibilityState(
            {
                routeId,
                sectionKeys,
                defaults,
                fieldName,
            },
            next,
        );
        setRevision((value) => value + 1);
    };

    return {
        state,
        setVisibility,
        toggle,
    };
}
