/**
 * Internationalization module - loads and provides translations from locales.json.
 * Uses i18next v4 JSON format with _one/_other suffixes for pluralization.
 * 
 * Usage:
 *   await translation.init();
 *   translation.get('books');           // → "Books"
 *   translation.get('pages', 5);        // → "5 pages"
 *   translation.get('pages', 1);        // → "1 page"
 *   translation.getLanguage();          // → "en-US"
 */

let data: Record<string, string> | null = null;
let currentLanguage: string = 'en-US';
let loadPromise: Promise<void> | null = null;

async function load(): Promise<void> {
    if (data) return;
    try {
        const res = await fetch('/assets/json/locales.json');
        const fullData = await res.json() as { language: string; translations: Record<string, string> };
        currentLanguage = fullData.language || 'en-US';
        data = fullData.translations || fullData; // Support both old and new format
    } catch (e) {
        console.warn('Failed to load translations:', e);
        data = {};
        currentLanguage = 'en-US';
    }
}

export const translation = {
    /**
     * Initialize translations. Call once at app startup.
     * Safe to call multiple times - will only load once.
     */
    async init(): Promise<void> {
        if (!loadPromise) {
            loadPromise = load();
        }
        await loadPromise;
    },

    /**
     * Get a translation by key, optionally with a count for pluralization.
     * Uses i18next format: looks up `key_one` or `key_other` based on count.
     * @param key - The translation key (e.g., 'pages', 'unknown-book')
     * @param count - Optional count for pluralized strings
     * @returns The translated string, or the key itself if not found
     */
    get(key: string, count?: number): string {
        if (!data) return key;

        // If count is provided, try pluralized keys first
        if (count !== undefined) {
            const suffix = count === 1 ? '_one' : '_other';
            const pluralKey = `${key}${suffix}`;
            const val = data[pluralKey];
            if (val) {
                return val.replace('{{ count }}', String(count));
            }
        }

        // Fall back to non-pluralized key, or 'other' variant if base is missing
        const val = data[key] || data[`${key}_other`];
        if (!val) return key;

        // Still replace {{ count }} if present and count was provided
        return count !== undefined
            ? val.replace('{{ count }}', String(count))
            : val;
    },

    /**
     * Get the locale in BCP 47 format (e.g., 'en-US', 'de-DE', 'pt-BR').
     */
    getLanguage(): string {
        return currentLanguage;
    }
};
