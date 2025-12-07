/**
 * Central LocalStorage Manager
 * Handles namespaced storage to avoid collisions and provides type safety helper.
 */
export class StorageManager {
    static PREFIX = 'koshelf_';

    // Centralized keys configuration
    static KEYS = {
        RECAP_SORT_ORDER: 'recap_sort_newest_first',
        // Add future keys here
    };

    /**
     * Get a value from local storage
     * @param {string} key - The key to retrieve (value from StorageManager.KEYS)
     * @param {any} defaultValue - Default value if key doesn't exist
     * @returns {any} - The stored value or default
     */
    static get(key, defaultValue = null) {
        try {
            const item = localStorage.getItem(this.PREFIX + key);
            if (item === null) return defaultValue;
            return JSON.parse(item);
        } catch (e) {
            console.warn('StorageManager: Failed to parse item', e);
            return defaultValue;
        }
    }

    /**
     * Set a value in local storage
     * @param {string} key - The key to set (value from StorageManager.KEYS)
     * @param {any} value - The value to store (will be JSON stringified)
     */
    static set(key, value) {
        try {
            localStorage.setItem(this.PREFIX + key, JSON.stringify(value));
        } catch (e) {
            console.warn('StorageManager: Failed to save item', e);
        }
    }

    /**
     * Remove a value from local storage
     * @param {string} key - The key to remove (value from StorageManager.KEYS)
     */
    static remove(key) {
        localStorage.removeItem(this.PREFIX + key);
    }
}
