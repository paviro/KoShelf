import likelySubtagsJson from 'cldr-core/supplemental/likelySubtags.json';
import territoryInfoJson from 'cldr-core/supplemental/territoryInfo.json';
import countries from 'i18n-iso-countries';

type LocaleFileModule = {
    default: string;
};

type LocaleFileLoader = () => Promise<LocaleFileModule>;

type LikelySubtagsPayload = {
    supplemental?: {
        likelySubtags?: Record<string, string>;
    };
};

type TerritoryInfoPayload = {
    supplemental?: {
        territoryInfo?: Record<
            string,
            {
                languagePopulation?: Record<
                    string,
                    { _officialStatus?: OfficialStatus }
                >;
            }
        >;
    };
};

type OfficialStatus = 'official' | 'de_facto_official' | 'official_regional';

type BaseLocaleMetadata = {
    code: string;
    dialect: string | null;
    nativeName: string | null;
    defaultRegion: string | null;
};

export type SupportedLanguageOption = {
    code: string;
    label: string;
    defaultRegion: string | null;
    dialect: string | null;
};

export type RegionOption = {
    code: string;
    label: string;
};

const localeModuleLoaders = import.meta.glob('../../../../locales/*.ftl', {
    query: '?raw',
}) as Record<string, LocaleFileLoader>;

const likelySubtags =
    (likelySubtagsJson as LikelySubtagsPayload).supplemental?.likelySubtags ??
    {};
const territoryInfo =
    (territoryInfoJson as TerritoryInfoPayload).supplemental?.territoryInfo ??
    {};

const ALL_REGION_CODES = Object.keys(countries.getAlpha2Codes())
    .map((code) => code.toUpperCase())
    .sort();

const ALL_REGION_CODE_SET = new Set(ALL_REGION_CODES);
const OFFICIAL_LANGUAGE_STATUSES: ReadonlySet<OfficialStatus> = new Set([
    'official',
    'de_facto_official',
    'official_regional',
]);
const OFFICIAL_REGION_CODES_BY_LANGUAGE = buildOfficialRegionCodesByLanguage();
const RUNTIME_SUPPORTED_REGION_CODES_BY_LANGUAGE = new Map<string, Set<string>>();
let baseLocaleMetadataPromise: Promise<BaseLocaleMetadata[]> | null = null;

function readMetadata(content: string, key: string): string | null {
    const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const match = content.match(new RegExp(`^\\s*${escapedKey}\\s*=\\s*(.+)$`, 'm'));
    if (!match) {
        return null;
    }

    return match[1]?.trim() || null;
}

async function loadBaseLocaleMetadata(): Promise<BaseLocaleMetadata[]> {
    const metadataEntries = await Promise.all(
        Object.entries(localeModuleLoaders).map(async ([modulePath, loader]) => {
            const fileName = modulePath.split('/').pop() ?? '';
            if (!fileName.endsWith('.ftl') || fileName.includes('_')) {
                return null;
            }

            const localeModule = await loader().catch(() => null);
            if (!localeModule) {
                return null;
            }

            const code = fileName.replace(/\.ftl$/i, '').toLowerCase();
            const dialect = readMetadata(localeModule.default, '-lang-dialect');
            const nativeName = readMetadata(localeModule.default, '-lang-name');

            return {
                code,
                dialect,
                nativeName,
                defaultRegion: extractRegion(dialect),
            };
        }),
    );

    return metadataEntries
        .filter((value): value is BaseLocaleMetadata => Boolean(value))
        .sort((left, right) => left.code.localeCompare(right.code));
}

function getBaseLocaleMetadata(): Promise<BaseLocaleMetadata[]> {
    if (!baseLocaleMetadataPromise) {
        baseLocaleMetadataPromise = loadBaseLocaleMetadata();
    }

    return baseLocaleMetadataPromise;
}

function createDisplayNames(
    locale: string,
    type: 'language' | 'region',
): Intl.DisplayNames | null {
    try {
        return new Intl.DisplayNames([locale || 'en-US'], { type });
    } catch {
        return null;
    }
}

function extractLanguage(value: string | null | undefined): string | null {
    if (!value) {
        return null;
    }

    const normalized = value.trim().replaceAll('_', '-');
    if (!normalized) {
        return null;
    }

    const [language] = normalized.split('-');
    if (!language) {
        return null;
    }

    return language.toLowerCase();
}

function extractRegion(value: string | null | undefined): string | null {
    if (!value) {
        return null;
    }

    const normalized = value.trim().replaceAll('_', '-');
    const parts = normalized.split('-');

    // Prefer an already-uppercase 2-letter part (canonical BCP 47 region subtag)
    // to avoid mistaking a lowercase language code (e.g. "de") for a region.
    for (const part of parts) {
        if (/^[A-Z]{2}$/.test(part) && ALL_REGION_CODE_SET.has(part)) {
            return part;
        }
    }

    // Fallback: accept case-insensitive 2-letter parts (e.g. bare "at" input).
    for (const part of parts) {
        if (/^[A-Za-z]{2}$/.test(part)) {
            const upper = part.toUpperCase();
            if (ALL_REGION_CODE_SET.has(upper)) {
                return upper;
            }
        }
    }

    return null;
}

function toRegionLabel(
    displayNames: Intl.DisplayNames | null,
    code: string,
): string {
    return displayNames?.of(code) ?? code;
}

function buildOfficialRegionCodesByLanguage(): Map<string, string[]> {
    const regionCodesByLanguage = new Map<string, Set<string>>();

    for (const [regionCode, regionInfo] of Object.entries(territoryInfo)) {
        const normalizedRegionCode = regionCode.toUpperCase();
        if (!ALL_REGION_CODE_SET.has(normalizedRegionCode)) {
            continue;
        }

        for (const [languageCode, languageInfo] of Object.entries(
            regionInfo.languagePopulation ?? {},
        )) {
            const normalizedLanguageCode = extractLanguage(languageCode);
            if (!normalizedLanguageCode) {
                continue;
            }

            if (
                !languageInfo._officialStatus ||
                !OFFICIAL_LANGUAGE_STATUSES.has(languageInfo._officialStatus)
            ) {
                continue;
            }

            const existingCodes = regionCodesByLanguage.get(normalizedLanguageCode);
            if (existingCodes) {
                existingCodes.add(normalizedRegionCode);
                continue;
            }

            regionCodesByLanguage.set(
                normalizedLanguageCode,
                new Set([normalizedRegionCode]),
            );
        }
    }

    return new Map(
        Array.from(regionCodesByLanguage.entries()).map(([languageCode, codes]) => [
            languageCode,
            Array.from(codes).sort(),
        ]),
    );
}

export function splitLocale(locale: string): {
    languageCode: string;
    regionCode: string | null;
} {
    const normalized = locale.trim().replaceAll('_', '-');
    const [language, ...rest] = normalized.split('-');
    const languageCode = language?.toLowerCase() || 'en';
    const regionCode = extractRegion(rest.join('-'));

    return { languageCode, regionCode };
}

export function joinLocale(
    languageCode: string,
    regionCode: string | null | undefined,
): string {
    const normalizedLanguage = extractLanguage(languageCode) || 'en';
    const normalizedRegion = extractRegion(regionCode);
    if (!normalizedRegion) {
        return normalizedLanguage;
    }

    return `${normalizedLanguage}-${normalizedRegion}`;
}

function isRuntimeLocaleRegionSupported(
    languageCode: string,
    regionCode: string,
): boolean {
    try {
        const resolvedLocale = new Intl.DateTimeFormat(
            joinLocale(languageCode, regionCode),
        ).resolvedOptions().locale;
        return extractRegion(resolvedLocale) === regionCode;
    } catch {
        return false;
    }
}

function getRuntimeSupportedRegionCodes(languageCode: string): Set<string> {
    const normalizedLanguage = extractLanguage(languageCode);
    if (!normalizedLanguage) {
        return new Set();
    }

    const cached = RUNTIME_SUPPORTED_REGION_CODES_BY_LANGUAGE.get(normalizedLanguage);
    if (cached) {
        return cached;
    }

    const supportedRegionCodes = new Set<string>();
    for (const regionCode of ALL_REGION_CODES) {
        if (isRuntimeLocaleRegionSupported(normalizedLanguage, regionCode)) {
            supportedRegionCodes.add(regionCode);
        }
    }

    RUNTIME_SUPPORTED_REGION_CODES_BY_LANGUAGE.set(
        normalizedLanguage,
        supportedRegionCodes,
    );

    return supportedRegionCodes;
}

export async function getSupportedLanguageOptions(): Promise<
    SupportedLanguageOption[]
> {
    const metadata = await getBaseLocaleMetadata();

    return metadata.map((entry) => {
        const baseName = entry.nativeName?.replace(/\s*\(.*\)$/, '') ?? null;
        const label = baseName
            ? `${entry.code.toUpperCase()} - ${baseName}`
            : entry.code.toUpperCase();

        return {
            code: entry.code,
            label,
            defaultRegion: entry.defaultRegion,
            dialect: entry.dialect,
        };
    }).sort((left, right) => left.code.localeCompare(right.code));
}

export function getLikelyRegionCodes(
    languageCode: string,
    preferredDefaultRegion?: string | null,
): string[] {
    const normalizedLanguage = extractLanguage(languageCode);
    if (!normalizedLanguage) {
        return [];
    }

    const likelyRegions = new Set<string>();

    for (const [fromLocale, toLocale] of Object.entries(likelySubtags)) {
        const fromLang = extractLanguage(fromLocale);
        const toLang = extractLanguage(toLocale);

        if (fromLang !== normalizedLanguage && toLang !== normalizedLanguage) {
            continue;
        }

        const fromRegion = extractRegion(fromLocale);
        if (fromRegion) {
            likelyRegions.add(fromRegion);
        }

        const toRegion = extractRegion(toLocale);
        if (toRegion) {
            likelyRegions.add(toRegion);
        }
    }

    const normalizedPreferredDefaultRegion = extractRegion(preferredDefaultRegion);
    if (normalizedPreferredDefaultRegion) {
        likelyRegions.add(normalizedPreferredDefaultRegion);
    }

    const officialRegionCodes =
        OFFICIAL_REGION_CODES_BY_LANGUAGE.get(normalizedLanguage) ?? [];
    for (const regionCode of officialRegionCodes) {
        likelyRegions.add(regionCode);
    }

    return Array.from(likelyRegions).sort();
}

export function getRegionOptionsForLanguage(
    languageCode: string,
    displayLocale: string,
    preferredDefaultRegion?: string | null,
): {
    likelyRegions: RegionOption[];
    allRegions: RegionOption[];
} {
    const normalizedLanguage = extractLanguage(languageCode) || 'en';
    const regionDisplayNames = createDisplayNames(displayLocale, 'region');
    const runtimeSupportedRegionCodes =
        getRuntimeSupportedRegionCodes(normalizedLanguage);
    const likelyCodes = new Set(
        getLikelyRegionCodes(
            normalizedLanguage,
            preferredDefaultRegion,
        ).filter((code) => runtimeSupportedRegionCodes.has(code)),
    );

    const allRegions = ALL_REGION_CODES.filter((code) =>
        runtimeSupportedRegionCodes.has(code),
    ).map((code) => ({
        code,
        label: `${code} - ${toRegionLabel(regionDisplayNames, code)}`,
    }));

    const likelyRegions = allRegions.filter((option) => likelyCodes.has(option.code));

    return {
        likelyRegions,
        allRegions,
    };
}
