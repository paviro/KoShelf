type CssModule = { default: string };

type FontDefinition = {
    family: string;
    importCss: () => Promise<CssModule>;
    aliases?: readonly string[];
};

type FontEntry = {
    family: string;
    loadCss: () => Promise<string>;
    aliases: readonly string[];
};

function createCssLoader(
    importCss: () => Promise<CssModule>,
): () => Promise<string> {
    let cssPromise: Promise<string> | null = null;

    return () => {
        if (!cssPromise) {
            cssPromise = importCss()
                .then((module) => module.default)
                .catch((error) => {
                    cssPromise = null;
                    throw error;
                });
        }

        return cssPromise;
    };
}

const FONT_DEFINITIONS: readonly FontDefinition[] = [
    {
        family: 'Noto Serif',
        importCss: () => import('@fontsource/noto-serif/400.css?inline'),
    },
    {
        family: 'Noto Sans',
        importCss: () => import('@fontsource/noto-sans/400.css?inline'),
    },
    {
        family: 'Noto Naskh Arabic',
        importCss: () => import('@fontsource/noto-naskh-arabic/400.css?inline'),
    },
    {
        family: 'Noto Sans Arabic',
        importCss: () => import('@fontsource/noto-sans-arabic/400.css?inline'),
        aliases: ['Noto Sans Arabic UI'],
    },
    {
        family: 'Noto Sans Hebrew',
        importCss: () => import('@fontsource/noto-sans-hebrew/400.css?inline'),
        aliases: ['Noto Sans Hebrew UI'],
    },
    {
        family: 'Noto Sans Devanagari',
        importCss: () =>
            import('@fontsource/noto-sans-devanagari/400.css?inline'),
        aliases: ['Noto Sans Devanagari UI'],
    },
    {
        family: 'Noto Sans Bengali',
        importCss: () => import('@fontsource/noto-sans-bengali/400.css?inline'),
        aliases: ['Noto Sans Bengali UI'],
    },
    {
        family: 'Noto Sans Thai',
        importCss: () => import('@fontsource/noto-sans-thai/400.css?inline'),
        aliases: ['Noto Sans Thai UI'],
    },
    {
        family: 'Noto Sans JP',
        importCss: () => import('@fontsource/noto-sans-jp/400.css?inline'),
        aliases: ['Noto Sans CJKjp'],
    },
    {
        family: 'Noto Sans KR',
        importCss: () => import('@fontsource/noto-sans-kr/400.css?inline'),
        aliases: ['Noto Sans CJKkr'],
    },
    {
        family: 'Noto Sans SC',
        importCss: () => import('@fontsource/noto-sans-sc/400.css?inline'),
        aliases: ['Noto Sans CJKsc'],
    },
    {
        family: 'Noto Sans TC',
        importCss: () => import('@fontsource/noto-sans-tc/400.css?inline'),
        aliases: ['Noto Sans CJKtc'],
    },
];

const FONT_REGISTRY: readonly FontEntry[] = FONT_DEFINITIONS.map(
    ({ family, importCss, aliases }) => ({
        family,
        loadCss: createCssLoader(importCss),
        aliases: aliases ?? [],
    }),
);

const DEFAULT_FALLBACK_FAMILY = 'Noto Serif';

const GENERIC_FONT_FAMILIES = new Set([
    'serif',
    'sans-serif',
    'monospace',
    'cursive',
    'fantasy',
    'system-ui',
    'ui-serif',
    'ui-sans-serif',
    'ui-monospace',
    'math',
    'fangsong',
    'emoji',
    'inherit',
    'initial',
    'unset',
]);

const FONT_REGISTRY_BY_KEY: Record<string, FontEntry> = {};
for (const entry of FONT_REGISTRY) {
    FONT_REGISTRY_BY_KEY[normalizeFontKey(entry.family)] = entry;
    for (const alias of entry.aliases) {
        FONT_REGISTRY_BY_KEY[normalizeFontKey(alias)] = entry;
    }
}

const DEFAULT_FALLBACK =
    FONT_REGISTRY_BY_KEY[normalizeFontKey(DEFAULT_FALLBACK_FAMILY)] ??
    FONT_REGISTRY[0];

function uniqueEntriesByFamily(entries: readonly FontEntry[]): FontEntry[] {
    const uniqueEntries: FontEntry[] = [];
    const seen = new Set<string>();

    for (const entry of entries) {
        const key = normalizeFontKey(entry.family);
        if (seen.has(key)) {
            continue;
        }

        seen.add(key);
        uniqueEntries.push(entry);
    }

    return uniqueEntries;
}

const GLOBAL_FALLBACK_ENTRIES = uniqueEntriesByFamily(FONT_REGISTRY);

function normalizeFontKey(value: string): string {
    return value.trim().replace(/\s+/g, ' ').toLowerCase();
}

function unquoteFontFamily(value: string): string {
    return value.replace(/^['"]+|['"]+$/g, '').trim();
}

function stripEmbeddedSuffix(value: string): string {
    return value.replace(/\bembedded\b\s*$/i, '').trim();
}

function isGenericFontFamily(value: string): boolean {
    return GENERIC_FONT_FAMILIES.has(normalizeFontKey(value));
}

function extractRequestedFontFamily(
    fontFace: string | null | undefined,
): string | null {
    const normalizedFace = stripEmbeddedSuffix(fontFace?.trim() ?? '');
    if (normalizedFace === '') {
        return null;
    }

    const families = normalizedFace
        .split(',')
        .map((family) => stripEmbeddedSuffix(unquoteFontFamily(family)))
        .map((family) => family.trim())
        .filter((family) => family !== '');

    const nonGenericFamily = families.find(
        (family) => !isGenericFontFamily(family),
    );
    if (nonGenericFamily) {
        return nonGenericFamily;
    }

    return null;
}

function resolvePackagedFont(
    family: string | null | undefined,
): FontEntry | undefined {
    if (!family) {
        return undefined;
    }

    return FONT_REGISTRY_BY_KEY[normalizeFontKey(family)];
}

function escapeCssString(value: string): string {
    return `'${value.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}'`;
}

function toFontFamilyCssValue(families: string[]): string {
    const orderedUniqueFamilies: string[] = [];
    const seenKeys = new Set<string>();

    for (const family of families) {
        const normalizedFamily = family.trim();
        if (normalizedFamily === '') {
            continue;
        }

        const key = normalizeFontKey(normalizedFamily);
        if (seenKeys.has(key)) {
            continue;
        }

        seenKeys.add(key);
        orderedUniqueFamilies.push(normalizedFamily);
    }

    return orderedUniqueFamilies
        .map((family) =>
            isGenericFontFamily(family) ? family : escapeCssString(family),
        )
        .join(', ');
}

function buildFontFaceCssKey(entries: FontEntry[]): string {
    return entries.map((entry) => normalizeFontKey(entry.family)).join('|');
}

const FONT_FACE_CSS_CACHE = new Map<string, Promise<string>>();
const FONT_OVERRIDE_CACHE = new Map<string, Promise<ResolvedReaderFont>>();

async function loadFontFaceCss(entries: FontEntry[]): Promise<string> {
    const cacheKey = buildFontFaceCssKey(entries);
    const cachedCss = FONT_FACE_CSS_CACHE.get(cacheKey);
    if (cachedCss) {
        return cachedCss;
    }

    const fontFaceCssPromise = Promise.all(
        entries.map((entry) => entry.loadCss()),
    )
        .then((chunks) => {
            const uniqueChunks = new Set<string>();
            for (const chunk of chunks) {
                if (typeof chunk !== 'string' || chunk.trim() === '') {
                    continue;
                }

                uniqueChunks.add(chunk);
            }

            return Array.from(uniqueChunks).join('\n');
        })
        .catch((error) => {
            FONT_FACE_CSS_CACHE.delete(cacheKey);
            throw error;
        });

    FONT_FACE_CSS_CACHE.set(cacheKey, fontFaceCssPromise);
    return fontFaceCssPromise;
}

export type ResolvedReaderFont = {
    requestedFamily: string;
    fallbackFamily: string;
    fontFamilyCssValue: string;
    fontFaceCss: string;
};

export async function resolveReaderFontOverride(
    fontFace: string | null | undefined,
): Promise<ResolvedReaderFont> {
    const cacheKey = fontFace?.trim() ?? '';
    const cachedOverride = FONT_OVERRIDE_CACHE.get(cacheKey);
    if (cachedOverride) {
        return cachedOverride;
    }

    const resolvedOverridePromise = (async (): Promise<ResolvedReaderFont> => {
        const extractedFamily = extractRequestedFontFamily(fontFace);
        const requestedEntry = resolvePackagedFont(extractedFamily);
        const requestedFamily =
            requestedEntry?.family ??
            extractedFamily ??
            DEFAULT_FALLBACK.family;
        const fallbackFamily = DEFAULT_FALLBACK.family;

        const fontStackEntries = uniqueEntriesByFamily(
            requestedEntry
                ? [requestedEntry, ...GLOBAL_FALLBACK_ENTRIES]
                : GLOBAL_FALLBACK_ENTRIES,
        );

        const fontFaceCss = await loadFontFaceCss(fontStackEntries);

        const fallbackFamilies = GLOBAL_FALLBACK_ENTRIES.map(
            (entry) => entry.family,
        );

        return {
            requestedFamily,
            fallbackFamily,
            fontFamilyCssValue: toFontFamilyCssValue([
                requestedFamily,
                ...fallbackFamilies,
                'serif',
            ]),
            fontFaceCss,
        };
    })().catch((error) => {
        FONT_OVERRIDE_CACHE.delete(cacheKey);
        throw error;
    });

    FONT_OVERRIDE_CACHE.set(cacheKey, resolvedOverridePromise);
    return resolvedOverridePromise;
}
