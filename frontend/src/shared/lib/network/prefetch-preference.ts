import { StorageManager } from '../../storage-manager';

const PREFETCH_STORAGE_KEY = 'prefetch_enabled';
const LEGACY_PREFETCH_STORAGE_KEY = 'prefetch_on_intent_enabled';

function normalizePrefetchOnIntentPreference(value: unknown): boolean {
    if (typeof value === 'boolean') {
        return value;
    }
    return true;
}

export function getPrefetchOnIntentPreference(): boolean {
    const currentValue = StorageManager.getByKey(PREFETCH_STORAGE_KEY, null);
    if (typeof currentValue === 'boolean') {
        return currentValue;
    }

    const legacyValue = StorageManager.getByKey(
        LEGACY_PREFETCH_STORAGE_KEY,
        null,
    );
    if (typeof legacyValue === 'boolean') {
        StorageManager.setByKey(PREFETCH_STORAGE_KEY, legacyValue);
        return legacyValue;
    }

    return true;
}

export function setPrefetchOnIntentPreference(enabled: boolean): void {
    StorageManager.setByKey(
        PREFETCH_STORAGE_KEY,
        normalizePrefetchOnIntentPreference(enabled),
    );
}
