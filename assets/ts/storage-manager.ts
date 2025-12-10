/**
 * Central LocalStorage Manager
 * Handles namespaced storage to avoid collisions and provides type safety.
 */

type StorageKey = typeof StorageManager.KEYS[keyof typeof StorageManager.KEYS];

export class StorageManager {
    static readonly PREFIX = 'koshelf_';

    static readonly KEYS = {
        RECAP_SORT_ORDER: 'recap_sort_newest_first',
        // PWA & Versioning
        VERSION: 'version',
        SERVER_MODE: 'server_mode',
        RELOAD_COUNT: 'reload_count',
        LAST_RELOAD: 'last_reload_time',
    } as const;

    /**
     * Get a value from local storage
     */
    static get<T>(key: StorageKey, defaultValue: T | null = null): T | null {
        try {
            const item = localStorage.getItem(this.PREFIX + key);
            if (item === null) return defaultValue;
            return JSON.parse(item) as T;
        } catch (e) {
            console.warn('StorageManager: Failed to parse item', e);
            return defaultValue;
        }
    }

    /**
     * Set a value in local storage
     */
    static set(key: StorageKey, value: unknown): void {
        try {
            localStorage.setItem(this.PREFIX + key, JSON.stringify(value));
        } catch (e) {
            console.warn('StorageManager: Failed to save item', e);
        }
    }

    /**
     * Remove a value from local storage
     */
    static remove(key: StorageKey): void {
        localStorage.removeItem(this.PREFIX + key);
    }
}
