import territoryInfoJson from 'cldr-core/supplemental/territoryInfo.json';
import countries from 'i18n-iso-countries';

type LocaleFileModule = {
    default: string;
};

type LocaleFileLoader = () => Promise<LocaleFileModule>;

type TerritoryInfoPayload = {
    supplemental?: {
        territoryInfo?: Record<
            string,
            {
                _population?: string;
                languagePopulation?: Record<
                    string,
                    {
                        _populationPercent?: string;
                        _officialStatus?: OfficialStatus;
                    }
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

type RegionLanguageCandidate = {
    languageCode: string;
    officialRank: number;
    populationPercent: number;
};

const localeModuleLoaders = import.meta.glob('../../../../locales/*.ftl', {
    query: '?raw',
}) as Record<string, LocaleFileLoader>;

const territoryInfo =
    (territoryInfoJson as TerritoryInfoPayload).supplemental?.territoryInfo ??
    {};

const ALL_REGION_CODES = Object.keys(countries.getAlpha2Codes())
    .map((code) => code.toUpperCase())
    .sort((left, right) => left.localeCompare(right));

const ALL_REGION_CODE_SET = new Set(ALL_REGION_CODES);
const MIN_LIKELY_LANGUAGE_SHARE_PERCENT = 60;
const RUNTIME_SUPPORTED_REGION_CODES_BY_LANGUAGE = new Map<
    string,
    Set<string>
>();
const REGION_PATTERN_LOCALE_BY_KEY = new Map<string, string | null>();
let baseLocaleMetadataPromise: Promise<BaseLocaleMetadata[]> | null = null;

function readMetadata(content: string, key: string): string | null {
    const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const match = content.match(
        new RegExp(`^\\s*${escapedKey}\\s*=\\s*(.+)$`, 'm'),
    );
    if (!match) {
        return null;
    }

    return match[1]?.trim() || null;
}

async function loadBaseLocaleMetadata(): Promise<BaseLocaleMetadata[]> {
    const metadataEntries = await Promise.all(
        Object.entries(localeModuleLoaders).map(
            async ([modulePath, loader]) => {
                const fileName = modulePath.split('/').pop() ?? '';
                if (!fileName.endsWith('.ftl') || fileName.includes('_')) {
                    return null;
                }

                const localeModule = await loader().catch(() => null);
                if (!localeModule) {
                    return null;
                }

                const code = fileName.replace(/\.ftl$/i, '').toLowerCase();
                const dialect = readMetadata(
                    localeModule.default,
                    '-lang-dialect',
                );
                const nativeName = readMetadata(
                    localeModule.default,
                    '-lang-name',
                );

                return {
                    code,
                    dialect,
                    nativeName,
                    defaultRegion: extractRegion(dialect),
                };
            },
        ),
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

    // Fallback: accept case-insensitive region subtags after the language
    // subtag (e.g. "de-at"), but do not treat bare language tags like "de"
    // or "fr" as if they were country codes.
    for (let index = 1; index < parts.length; index += 1) {
        const part = parts[index];
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

function getOfficialStatusRank(
    status: OfficialStatus | null | undefined,
): number {
    switch (status) {
        case 'official':
        case 'de_facto_official':
            return 3;
        case 'official_regional':
            return 2;
        default:
            return 0;
    }
}

function compareRegionLanguageCandidates(
    left: RegionLanguageCandidate,
    right: RegionLanguageCandidate,
): number {
    return (
        right.officialRank - left.officialRank ||
        right.populationPercent - left.populationPercent ||
        left.languageCode.localeCompare(right.languageCode)
    );
}

function supportsDatePatternLocale(
    locale: string,
    regionCode: string,
): boolean {
    try {
        const resolvedLocale = new Intl.DateTimeFormat(locale).resolvedOptions()
            .locale;
        return extractRegion(resolvedLocale) === regionCode;
    } catch {
        return false;
    }
}

function getRegionLanguageCandidates(
    regionCode: string,
): RegionLanguageCandidate[] {
    const regionInfo = territoryInfo[regionCode];
    if (!regionInfo) {
        return [];
    }

    const candidateByLanguage = new Map<string, RegionLanguageCandidate>();
    for (const [candidateLanguage, languageInfo] of Object.entries(
        regionInfo.languagePopulation ?? {},
    )) {
        const languageCode = extractLanguage(candidateLanguage);
        if (!languageCode) {
            continue;
        }

        const candidate: RegionLanguageCandidate = {
            languageCode,
            officialRank: getOfficialStatusRank(languageInfo._officialStatus),
            populationPercent: Number(languageInfo._populationPercent ?? 0),
        };
        const existing = candidateByLanguage.get(languageCode);
        if (
            !existing ||
            compareRegionLanguageCandidates(existing, candidate) > 0
        ) {
            candidateByLanguage.set(languageCode, candidate);
        }
    }

    return [...candidateByLanguage.values()].sort(
        compareRegionLanguageCandidates,
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

export function getRegionPatternLocale(
    regionCode: string | null | undefined,
    preferredLanguageCode?: string | null,
): string | null {
    const normalizedRegion = extractRegion(regionCode);
    if (!normalizedRegion) {
        return null;
    }

    const normalizedPreferredLanguage = extractLanguage(preferredLanguageCode);
    const cacheKey = `${normalizedRegion}:${normalizedPreferredLanguage ?? ''}`;
    if (REGION_PATTERN_LOCALE_BY_KEY.has(cacheKey)) {
        return REGION_PATTERN_LOCALE_BY_KEY.get(cacheKey) ?? null;
    }

    const candidates = getRegionLanguageCandidates(normalizedRegion);

    if (normalizedPreferredLanguage) {
        const preferredCandidate = candidates.find(
            (candidate) =>
                candidate.languageCode === normalizedPreferredLanguage &&
                candidate.officialRank > 0,
        );
        if (preferredCandidate) {
            const preferredLocale = joinLocale(
                normalizedPreferredLanguage,
                normalizedRegion,
            );
            if (supportsDatePatternLocale(preferredLocale, normalizedRegion)) {
                REGION_PATTERN_LOCALE_BY_KEY.set(cacheKey, preferredLocale);
                return preferredLocale;
            }
        }
    }

    for (const candidate of candidates) {
        const locale = joinLocale(candidate.languageCode, normalizedRegion);
        if (supportsDatePatternLocale(locale, normalizedRegion)) {
            REGION_PATTERN_LOCALE_BY_KEY.set(cacheKey, locale);
            return locale;
        }
    }

    const fallbackLocale = normalizedPreferredLanguage
        ? joinLocale(normalizedPreferredLanguage, normalizedRegion)
        : null;
    if (
        fallbackLocale &&
        supportsDatePatternLocale(fallbackLocale, normalizedRegion)
    ) {
        REGION_PATTERN_LOCALE_BY_KEY.set(cacheKey, fallbackLocale);
        return fallbackLocale;
    }

    REGION_PATTERN_LOCALE_BY_KEY.set(cacheKey, null);
    return null;
}

export function resolveLocalePatternContext(locale: string): {
    valueLocale: string;
    patternLocale: string | null;
} {
    const { languageCode, regionCode } = splitLocale(locale);
    const patternLocale = getRegionPatternLocale(regionCode, languageCode);
    if (!patternLocale) {
        return {
            valueLocale: locale,
            patternLocale: null,
        };
    }

    const patternParts = splitLocale(patternLocale);
    if (
        patternParts.languageCode === languageCode &&
        patternParts.regionCode === regionCode
    ) {
        return {
            valueLocale: locale,
            patternLocale: null,
        };
    }

    return {
        valueLocale: locale,
        patternLocale,
    };
}

function getRuntimeSupportedRegionCodes(languageCode: string): Set<string> {
    const normalizedLanguage = extractLanguage(languageCode);
    if (!normalizedLanguage) {
        return new Set();
    }

    const cached =
        RUNTIME_SUPPORTED_REGION_CODES_BY_LANGUAGE.get(normalizedLanguage);
    if (cached) {
        return cached;
    }

    const supportedRegionCodes = new Set<string>();
    for (const regionCode of ALL_REGION_CODES) {
        const locale = joinLocale(normalizedLanguage, regionCode);

        try {
            const resolvedDateLocale = new Intl.DateTimeFormat(
                locale,
            ).resolvedOptions().locale;
            if (extractRegion(resolvedDateLocale) === regionCode) {
                supportedRegionCodes.add(regionCode);
                continue;
            }
        } catch {
            // Ignore unsupported locale/region pairs.
        }

        try {
            const resolvedNumberLocale = new Intl.NumberFormat(
                locale,
            ).resolvedOptions().locale;
            if (extractRegion(resolvedNumberLocale) === regionCode) {
                supportedRegionCodes.add(regionCode);
            }
        } catch {
            // Ignore unsupported locale/region pairs.
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

    return metadata
        .map((entry) => {
            const baseName =
                entry.nativeName?.replace(/\s*\(.*\)$/, '') ?? null;
            const label = baseName
                ? `${entry.code.toUpperCase()} - ${baseName}`
                : entry.code.toUpperCase();

            return {
                code: entry.code,
                label,
                defaultRegion: entry.defaultRegion,
                dialect: entry.dialect,
            };
        })
        .sort((left, right) => left.code.localeCompare(right.code));
}

export function getLikelyRegionCodes(
    languageCode: string,
    preferredDefaultRegion?: string | null,
): string[] {
    const normalizedLanguage = extractLanguage(languageCode);
    if (!normalizedLanguage) {
        return [];
    }

    const normalizedPreferredDefaultRegion = extractRegion(
        preferredDefaultRegion,
    );
    const rankedLikelyRegions = Object.entries(territoryInfo)
        .map(([regionCode, regionInfo]) => {
            const normalizedRegionCode = regionCode.toUpperCase();
            if (!ALL_REGION_CODE_SET.has(normalizedRegionCode)) {
                return null;
            }

            const languageInfo = Object.entries(
                regionInfo.languagePopulation ?? {},
            ).find(
                ([candidateLanguage]) =>
                    extractLanguage(candidateLanguage) === normalizedLanguage,
            )?.[1];
            if (!languageInfo) {
                return null;
            }

            if (
                languageInfo._officialStatus !== 'official' &&
                languageInfo._officialStatus !== 'de_facto_official'
            ) {
                return null;
            }

            const populationPercent = Number(
                languageInfo._populationPercent ?? 0,
            );
            if (populationPercent < MIN_LIKELY_LANGUAGE_SHARE_PERCENT) {
                return null;
            }

            const population = Number(regionInfo._population ?? 0);

            return {
                code: normalizedRegionCode,
                score: populationPercent * Math.log10(Math.max(population, 10)),
                populationPercent,
                population,
            };
        })
        .filter(
            (
                entry,
            ): entry is {
                code: string;
                score: number;
                populationPercent: number;
                population: number;
            } => Boolean(entry),
        )
        .sort(
            (left, right) =>
                right.score - left.score ||
                right.populationPercent - left.populationPercent ||
                right.population - left.population ||
                left.code.localeCompare(right.code),
        );

    const likelyRegions: string[] = [];
    const seen = new Set<string>();

    if (normalizedPreferredDefaultRegion) {
        likelyRegions.push(normalizedPreferredDefaultRegion);
        seen.add(normalizedPreferredDefaultRegion);
    }

    for (const region of rankedLikelyRegions) {
        if (seen.has(region.code)) {
            continue;
        }

        seen.add(region.code);
        likelyRegions.push(region.code);
    }

    return likelyRegions;
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
        getLikelyRegionCodes(normalizedLanguage, preferredDefaultRegion).filter(
            (code) => runtimeSupportedRegionCodes.has(code),
        ),
    );

    const allRegions = ALL_REGION_CODES.filter((code) =>
        runtimeSupportedRegionCodes.has(code),
    ).map((code) => ({
        code,
        label: `${code} - ${toRegionLabel(regionDisplayNames, code)}`,
    }));

    const likelyRegions = allRegions.filter((option) =>
        likelyCodes.has(option.code),
    );

    return {
        likelyRegions,
        allRegions,
    };
}
