import { FluentBundle, FluentResource } from '@fluent/bundle';
import type { FluentVariable } from '@fluent/bundle';

let bundle: FluentBundle | null = null;
let loadPromise: Promise<void> | null = null;
let loadedLanguage = '';

const FALLBACK_LANGUAGE = 'en-US';
const LOCALE_STORAGE_KEY = 'koshelf_locale';
export const I18N_LANGUAGE_CHANGE_EVENT = 'koshelf:language-changed';
const localeModules = import.meta.glob('../../locales/*.ftl', {
    query: '?raw',
}) as Record<string, () => Promise<{ default: string }>>;
const localeLoaders = new Map<string, () => Promise<{ default: string }>>();
const localeContentCache = new Map<string, Promise<string | null>>();

for (const [modulePath, loader] of Object.entries(localeModules)) {
    const fileName = modulePath.split('/').pop();
    if (!fileName) continue;
    localeLoaders.set(fileName, loader);
}

function normalizeLanguage(language: string | undefined): string {
    const trimmed = language?.trim();
    if (!trimmed) return FALLBACK_LANGUAGE;
    return trimmed.replaceAll('_', '-');
}

function readStoredLanguage(): string | null {
    try {
        return localStorage.getItem(LOCALE_STORAGE_KEY);
    } catch {
        return null;
    }
}

function writeStoredLanguage(language: string): void {
    try {
        localStorage.setItem(LOCALE_STORAGE_KEY, language);
    } catch {
        // Ignore storage write failures.
    }
}

function hasSupportedLocale(language: string): boolean {
    const normalized = normalizeLanguage(language);
    const [base] = normalized.split('-');
    if (!base) return false;

    const baseFile = `${base}.ftl`;
    return localeLoaders.has(baseFile);
}

function emitLanguageChanged(): void {
    if (typeof window === 'undefined') {
        return;
    }

    window.dispatchEvent(new Event(I18N_LANGUAGE_CHANGE_EVENT));
}

function resolveRequestedLanguage(serverLanguage?: string): string {
    const normalizedDefault = normalizeLanguage(serverLanguage);
    const storedLanguage = readStoredLanguage();
    const normalizedStored = storedLanguage
        ? normalizeLanguage(storedLanguage)
        : null;

    if (normalizedStored && hasSupportedLocale(normalizedStored)) {
        return normalizedStored;
    }

    if (hasSupportedLocale(normalizedDefault)) {
        if (serverLanguage) {
            writeStoredLanguage(normalizedDefault);
        }
        return normalizedDefault;
    }

    return FALLBACK_LANGUAGE;
}

function selectResourceChainFiles(language: string): {
    language: string;
    files: string[];
} {
    const normalized = normalizeLanguage(language);
    const parts = normalized.split('-');
    const base = parts[0]?.toLowerCase();

    if (!base) {
        return { language: FALLBACK_LANGUAGE, files: [] };
    }

    const files: string[] = [];
    const region = parts.slice(1).join('_').toUpperCase();
    const regionalFile = region ? `${base}_${region}.ftl` : null;
    const baseFile = `${base}.ftl`;

    if (regionalFile) {
        files.push(regionalFile);
    }
    files.push(baseFile);
    if (base !== 'en') {
        files.push('en.ftl');
    }

    return { language: normalized, files };
}

async function loadLocaleFile(fileName: string): Promise<string | null> {
    const loader = localeLoaders.get(fileName);
    if (!loader) {
        return null;
    }

    let pending = localeContentCache.get(fileName);
    if (!pending) {
        pending = loader()
            .then((mod) => mod.default)
            .catch(() => null);
        localeContentCache.set(fileName, pending);
    }

    return pending;
}

async function selectResourceChain(
    language: string,
): Promise<{ language: string; resources: string[] }> {
    const selected = selectResourceChainFiles(language);
    const resources: string[] = [];
    for (const fileName of selected.files) {
        const content = await loadLocaleFile(fileName);
        if (content) {
            resources.push(content);
        }
    }

    return { language: selected.language, resources };
}

async function buildFallbackBundle(): Promise<FluentBundle> {
    const fallbackBundle = new FluentBundle(FALLBACK_LANGUAGE);
    const englishResource = await loadLocaleFile('en.ftl');
    if (englishResource) {
        fallbackBundle.addResource(new FluentResource(englishResource));
    }
    return fallbackBundle;
}

async function load(language: string): Promise<void> {
    if (bundle && loadedLanguage === language) return;

    try {
        const selected = await selectResourceChain(language);
        if (selected.resources.length === 0) {
            bundle = await buildFallbackBundle();
            loadedLanguage = FALLBACK_LANGUAGE;
            return;
        }

        bundle = new FluentBundle(selected.language);

        for (const resourceContent of selected.resources) {
            const resource = new FluentResource(resourceContent);
            bundle.addResource(resource);
        }
        loadedLanguage = selected.language;
    } catch {
        bundle = await buildFallbackBundle();
        loadedLanguage = FALLBACK_LANGUAGE;
    }
}

export const translation = {
    async init(language?: string): Promise<void> {
        const previousLanguage = loadedLanguage;
        const requestedLanguage = resolveRequestedLanguage(language);
        if (!loadPromise || requestedLanguage !== loadedLanguage) {
            loadPromise = load(requestedLanguage);
        }
        await loadPromise;
        if (loadedLanguage !== previousLanguage) {
            emitLanguageChanged();
        }
    },

    async setLanguage(language: string): Promise<void> {
        const normalized = normalizeLanguage(language);
        if (!hasSupportedLocale(normalized)) {
            return;
        }
        writeStoredLanguage(normalized);
        const previousLanguage = loadedLanguage;
        loadPromise = load(normalized);
        await loadPromise;
        if (loadedLanguage !== previousLanguage) {
            emitLanguageChanged();
        }
    },

    get(key: string, args?: number | Record<string, FluentVariable>): string {
        if (!bundle) return key;

        let fluentArgs: Record<string, FluentVariable> | undefined;
        if (typeof args === 'number') {
            fluentArgs = { count: args };
        } else {
            fluentArgs = args;
        }

        let messageId = key;
        let attributeId: string | undefined;

        const dotIndex = key.indexOf('.');
        if (dotIndex !== -1) {
            messageId = key.substring(0, dotIndex);
            attributeId = key.substring(dotIndex + 1);
        }

        const message = bundle.getMessage(messageId);
        if (!message) return key;

        const pattern = attributeId
            ? message.attributes?.[attributeId]
            : message.value;
        if (!pattern) return key;

        return bundle.formatPattern(pattern, fluentArgs);
    },

    getLanguage(): string {
        return bundle?.locales[0] || FALLBACK_LANGUAGE;
    },
};
