import type { RouteId } from '../../../app/routes/route-registry';
import { StorageManager } from '../../storage-manager';

export type RouteStateStorage = 'local' | 'session';
export type RouteStateRecord = Record<string, unknown>;

const ROUTE_STATE_KEY_PREFIX = 'route_state_';

function routeStateStorageKey(routeId: RouteId): string {
    return `${ROUTE_STATE_KEY_PREFIX}${routeId}`;
}

function normalizeStateRecord(value: unknown): RouteStateRecord {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        return {};
    }

    return value as RouteStateRecord;
}

function readRawRouteState(
    routeId: RouteId,
    storage: RouteStateStorage,
): RouteStateRecord {
    const key = routeStateStorageKey(routeId);
    const value =
        storage === 'local'
            ? StorageManager.getByKey<RouteStateRecord>(key)
            : StorageManager.getSessionByKey<RouteStateRecord>(key);
    return normalizeStateRecord(value);
}

function writeRawRouteState(
    routeId: RouteId,
    storage: RouteStateStorage,
    value: RouteStateRecord,
): void {
    const key = routeStateStorageKey(routeId);
    if (storage === 'local') {
        StorageManager.setByKey(key, value);
        return;
    }

    StorageManager.setSessionByKey(key, value);
}

export function readRouteState(
    routeId: RouteId,
    storage: RouteStateStorage,
): RouteStateRecord {
    return readRawRouteState(routeId, storage);
}

export function patchRouteState(
    routeId: RouteId,
    storage: RouteStateStorage,
    patch: RouteStateRecord,
): void {
    const current = readRawRouteState(routeId, storage);
    writeRawRouteState(routeId, storage, {
        ...current,
        ...normalizeStateRecord(patch),
    });
}
