/**
 * Central LocalStorage Manager
 * Handles namespaced storage to avoid collisions and provides type safety.
 */

type StorageKey = typeof StorageManager.KEYS[keyof typeof StorageManager.KEYS];

export class StorageManager {
    static readonly PREFIX = 'koshelf_';

    static readonly KEYS = {
        RECAP_SORT_ORDER: 'recap_sort_newest_first',
        RECAP_SCOPE: 'recap_scope',
        STATS_SCOPE: 'stats_scope',
        // List filters
        BOOK_LIST_FILTER: 'book_list_filter',
        COMIC_LIST_FILTER: 'comic_list_filter',
        // UI state
        BOOK_LIST_SECTION_STATE_BOOKS: 'book_list_section_state_books',
        BOOK_LIST_SECTION_STATE_COMICS: 'book_list_section_state_comics',
        BOOK_DETAILS_SECTION_STATE_BOOKS: 'book_details_section_state_books',
        BOOK_DETAILS_SECTION_STATE_COMICS: 'book_details_section_state_comics',
        STATISTICS_SECTION_STATE_ALL: 'statistics_section_state_all',
        STATISTICS_SECTION_STATE_BOOKS: 'statistics_section_state_books',
        STATISTICS_SECTION_STATE_COMICS: 'statistics_section_state_comics',
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
