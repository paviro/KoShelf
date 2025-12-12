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
        return this._resolveValue(key, count, 0);
    },

    /**
     * Internal recursive resolver for values.
     * Handles message references ({@ref:key}) by recursively looking them up
     * and passing the *same* count variable to them.
     */
    _resolveValue(key: string, count: number | undefined, depth: number): string {
        if (!data) return key;
        // Prevent infinite recursion
        if (depth > 10) {
            return `[recursion limit: ${key}]`;
        }

        let val: string | undefined;

        // 1. Lookup the raw value (Simple or Plural)
        if (count !== undefined) {
            const pluralRules = new Intl.PluralRules(currentLanguage);
            const category = pluralRules.select(count);
            val = data[`${key}_${category}`];

            if (!val) {
                // Fallback to 'other'
                val = data[`${key}_other`];
            }
        }

        // Fallback to simple key if no plural found or no count provided
        // or check if the simple key exists directly
        if (!val) {
            val = data[key] || data[`${key}_other`];
        }

        if (!val) return key;

        // 2. Resolve any nested message references {@ref:key}
        // This regex must match the format produced by the Rust backend
        if (val.includes('{@ref:')) {
            val = val.replace(/\{@ref:([^}]+)\}/g, (_match, refKey) => {
                return this._resolveValue(refKey, count, depth + 1);
            });
        }

        // 3. Replace {$count} placeholders
        if (count !== undefined) {
            return val.replace(/\{\s*\$count\s*\}/g, String(count));
        }

        return val;
    },

    /**
     * Get the locale in BCP 47 format (e.g., 'en-US', 'de-DE', 'pt-BR').
     */
    getLanguage(): string {
        return currentLanguage;
    }
};
