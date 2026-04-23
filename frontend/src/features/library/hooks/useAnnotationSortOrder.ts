import { useState } from 'react';

import type { RouteId } from '../../../app/routes/route-registry';
import {
    patchRouteState,
    readRouteState,
} from '../../../shared/lib/state/route-state-storage';

export type AnnotationSortOrder = 'asc' | 'desc';

type Options = {
    routeId: RouteId;
    sectionKey: string;
    defaultOrder?: AnnotationSortOrder;
};

const FIELD_NAME = 'annotationSort';

function isOrder(value: unknown): value is AnnotationSortOrder {
    return value === 'asc' || value === 'desc';
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
    return !!value && typeof value === 'object' && !Array.isArray(value);
}

export function readAnnotationSortOrder(
    routeId: RouteId,
    sectionKey: string,
    defaultOrder: AnnotationSortOrder,
): AnnotationSortOrder {
    const persisted = readRouteState(routeId, 'local')[FIELD_NAME];
    if (!isPlainObject(persisted)) {
        return defaultOrder;
    }
    const raw = persisted[sectionKey];
    return isOrder(raw) ? raw : defaultOrder;
}

export function writeAnnotationSortOrder(
    routeId: RouteId,
    sectionKey: string,
    order: AnnotationSortOrder,
): void {
    const existing = readRouteState(routeId, 'local')[FIELD_NAME];
    const base = isPlainObject(existing) ? existing : {};
    patchRouteState(routeId, 'local', {
        [FIELD_NAME]: { ...base, [sectionKey]: order },
    });
}

export function useAnnotationSortOrder({
    routeId,
    sectionKey,
    defaultOrder = 'asc',
}: Options): { order: AnnotationSortOrder; toggle: () => void } {
    const [, setRevision] = useState(0);
    const order = readAnnotationSortOrder(routeId, sectionKey, defaultOrder);

    const toggle = (): void => {
        const next: AnnotationSortOrder = order === 'asc' ? 'desc' : 'asc';
        writeAnnotationSortOrder(routeId, sectionKey, next);
        setRevision((value) => value + 1);
    };

    return { order, toggle };
}
