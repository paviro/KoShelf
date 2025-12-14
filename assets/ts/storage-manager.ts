/**
 * Central LocalStorage Manager
 * Handles namespaced storage to avoid collisions and provides type safety.
 */

type StorageKey = typeof StorageManager.KEYS[keyof typeof StorageManager.KEYS];

export class StorageManager {
    static readonly PREFIX = 'koshelf_';

    static readonly KEYS = {
        // Filters (content type filtering)
        RECAP_FILTER: 'recap_filter',
        STATS_FILTER: 'stats_filter',
        BOOKS_LIST_FILTER: 'books_list_filter',
        COMICS_LIST_FILTER: 'comics_list_filter',
        // Section collapse states
        BOOKS_LIST_SECTIONS: 'books_list_sections',
        COMICS_LIST_SECTIONS: 'comics_list_sections',
        BOOK_DETAIL_SECTIONS: 'book_detail_sections',
        COMIC_DETAIL_SECTIONS: 'comic_detail_sections',
        STATS_ALL_SECTIONS: 'stats_all_sections',
        STATS_BOOKS_SECTIONS: 'stats_books_sections',
        STATS_COMICS_SECTIONS: 'stats_comics_sections',
        // Recap settings
        RECAP_SORT_NEWEST: 'recap_sort_newest',
        // PWA & Versioning
        VERSION: 'version',
        SERVER_MODE: 'server_mode',
        RELOAD_COUNT: 'reload_count',
        LAST_RELOAD: 'last_reload',
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
