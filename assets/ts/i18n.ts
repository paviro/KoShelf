/**
 * Internationalization module - loads and provides translations from locales.json.
 * Uses CLDR plural rules via Intl.PluralRules, with suffixes: _zero, _one, _two, _few, _many, _other.
 * 
 * Usage:
 *   await translation.init();
 *   translation.get('books');           // → "Books"
 *   translation.get('pages', 5);        // → "5 pages"
 *   translation.get('pages', 1);        // → "1 page"
 *   translation.get('share.recap-label'); // → "Share Recap Image" (attribute syntax)
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
     * Uses Intl.PluralRules for CLDR-compliant plural category selection.
     * Looks up suffixed keys: `key_zero`, `key_one`, `key_two`, `key_few`, `key_many`, `key_other`
     * @param key - The translation key (e.g., 'pages', 'unknown-book')
     * @param count - Optional count for pluralized strings
     * @returns The translated string, or the key itself if not found
     */
    get(key: string, count?: number): string {
        if (!data) return key;

        // If count is provided, use CLDR plural rules
        if (count !== undefined) {
            const pluralRules = new Intl.PluralRules(currentLanguage);
            const category = pluralRules.select(count); // Returns: 'zero', 'one', 'two', 'few', 'many', or 'other'
            const pluralKey = `${key}_${category}`;
            const val = data[pluralKey];
            if (val) {
                // Replace both Fluent placeholder formats: {$count} and { $count }
                return val.replace(/\{\s*\$count\s*\}/g, String(count));
            }
            // Fall back to 'other' if specific category not found
            const otherVal = data[`${key}_other`];
            if (otherVal) {
                return otherVal.replace(/\{\s*\$count\s*\}/g, String(count));
            }
        }

        // Fall back to non-pluralized key, or 'other' variant if base is missing
        const val = data[key] || data[`${key}_other`];
        if (!val) return key;

        // Still replace { $count } if present and count was provided
        return count !== undefined
            ? val.replace(/\{\s*\$count\s*\}/g, String(count))
            : val;
    },

    /**
     * Get the locale in BCP 47 format (e.g., 'en-US', 'de-DE', 'pt-BR').
     */
    getLanguage(): string {
        return currentLanguage;
    }
};
