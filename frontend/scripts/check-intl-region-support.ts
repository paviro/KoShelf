import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

type LikelySubtagsPayload = {
    supplemental?: {
        likelySubtags?: Record<string, string>;
    };
};

type OfficialStatus = 'official' | 'de_facto_official' | 'official_regional';

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

type LocaleMetadata = {
    languageCode: string;
    defaultRegion: string | null;
};

type LocaleSupportResult = {
    requested: string;
    resolved: string;
    retainsRequestedRegion: boolean;
    acceptedBySupportedLocalesOf: boolean;
};

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const localesDir = path.resolve(__dirname, '..', 'locales');
const require = createRequire(import.meta.url);

const likelySubtags =
    (require('cldr-core/supplemental/likelySubtags.json') as LikelySubtagsPayload)
        .supplemental?.likelySubtags ?? {};

const territoryInfo =
    (require('cldr-core/supplemental/territoryInfo.json') as TerritoryInfoPayload)
        .supplemental?.territoryInfo ?? {};

const countries = require('i18n-iso-countries') as typeof import('i18n-iso-countries');

const ALL_REGION_CODES = Object.keys(countries.getAlpha2Codes())
    .map((code) => code.toUpperCase())
    .sort();

const ALL_REGION_CODE_SET = new Set(ALL_REGION_CODES);
const OFFICIAL_STATUSES: ReadonlySet<OfficialStatus> = new Set([
    'official',
    'de_facto_official',
    'official_regional',
]);

function readMetadata(content: string, key: string): string | null {
    const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const match = content.match(new RegExp(`^\\s*${escapedKey}\\s*=\\s*(.+)$`, 'm'));
    if (!match) {
        return null;
    }
    return match[1]?.trim() || null;
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
    return language ? language.toLowerCase() : null;
}

function extractRegion(value: string | null | undefined): string | null {
    if (!value) {
        return null;
    }

    const parts = value.trim().replaceAll('_', '-').split('-');

    for (const part of parts) {
        if (/^[A-Z]{2}$/.test(part) && ALL_REGION_CODE_SET.has(part)) {
            return part;
        }
    }

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
                !OFFICIAL_STATUSES.has(languageInfo._officialStatus)
            ) {
                continue;
            }

            const existingCodes = regionCodesByLanguage.get(normalizedLanguageCode);
            if (existingCodes) {
                existingCodes.add(normalizedRegionCode);
            } else {
                regionCodesByLanguage.set(
                    normalizedLanguageCode,
                    new Set([normalizedRegionCode]),
                );
            }
        }
    }

    return new Map(
        Array.from(regionCodesByLanguage.entries()).map(([languageCode, codes]) => [
            languageCode,
            Array.from(codes).sort(),
        ]),
    );
}

function readSupportedLanguagesFromLocales(): LocaleMetadata[] {
    const localeFiles = fs
        .readdirSync(localesDir, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith('.ftl'))
        .map((entry) => entry.name)
        .filter((fileName) => !fileName.includes('_'))
        .sort();

    return localeFiles.map((fileName) => {
        const content = fs.readFileSync(path.join(localesDir, fileName), 'utf8');
        const languageCode =
            readMetadata(content, '-lang-code') ||
            fileName.replace(/\.ftl$/i, '').toLowerCase();
        const dialect = readMetadata(content, '-lang-dialect');

        return {
            languageCode: extractLanguage(languageCode) || 'en',
            defaultRegion: extractRegion(dialect),
        };
    });
}

function getSuggestedRegionCodes(
    languageCode: string,
    defaultRegion: string | null,
    officialByLanguage: Map<string, string[]>,
): string[] {
    const normalizedLanguage = extractLanguage(languageCode);
    if (!normalizedLanguage) {
        return [];
    }

    const suggested = new Set<string>();

    for (const [fromLocale, toLocale] of Object.entries(likelySubtags)) {
        const fromLanguage = extractLanguage(fromLocale);
        const toLanguage = extractLanguage(toLocale);
        if (fromLanguage !== normalizedLanguage && toLanguage !== normalizedLanguage) {
            continue;
        }

        const fromRegion = extractRegion(fromLocale);
        if (fromRegion) {
            suggested.add(fromRegion);
        }

        const toRegion = extractRegion(toLocale);
        if (toRegion) {
            suggested.add(toRegion);
        }
    }

    if (defaultRegion) {
        suggested.add(defaultRegion);
    }

    for (const regionCode of officialByLanguage.get(normalizedLanguage) ?? []) {
        suggested.add(regionCode);
    }

    return Array.from(suggested).sort();
}

function evaluateLocaleSupport(locale: string): LocaleSupportResult {
    const requestedRegion = extractRegion(locale);
    const acceptedBySupportedLocalesOf =
        Intl.DateTimeFormat.supportedLocalesOf([locale]).length > 0;

    let resolved = 'n/a';
    let resolvedRegion: string | null = null;

    try {
        resolved = new Intl.DateTimeFormat(locale).resolvedOptions().locale;
        resolvedRegion = extractRegion(resolved);
    } catch {
        // Keep resolved as "n/a".
    }

    return {
        requested: locale,
        resolved,
        retainsRequestedRegion:
            Boolean(requestedRegion) && requestedRegion === resolvedRegion,
        acceptedBySupportedLocalesOf,
    };
}

function main(): void {
    const supportedLanguages = readSupportedLanguagesFromLocales();
    const officialByLanguage = buildOfficialRegionCodesByLanguage();

    let totalCombos = 0;
    let retainedRegionCount = 0;
    let acceptedCount = 0;

    console.log('Intl locale-region support check');
    console.log('--------------------------------');

    for (const language of supportedLanguages) {
        const regionCodes = getSuggestedRegionCodes(
            language.languageCode,
            language.defaultRegion,
            officialByLanguage,
        );

        const dropped: LocaleSupportResult[] = [];
        let retainedForLanguage = 0;
        let acceptedForLanguage = 0;

        for (const regionCode of regionCodes) {
            const requested = `${language.languageCode}-${regionCode}`;
            const result = evaluateLocaleSupport(requested);

            totalCombos += 1;
            if (result.acceptedBySupportedLocalesOf) {
                acceptedCount += 1;
                acceptedForLanguage += 1;
            }

            if (result.retainsRequestedRegion) {
                retainedRegionCount += 1;
                retainedForLanguage += 1;
            } else {
                dropped.push(result);
            }
        }

        console.log(
            `${language.languageCode.toUpperCase()}: ` +
                `${retainedForLanguage}/${regionCodes.length} keep region ` +
                `(${acceptedForLanguage} accepted by supportedLocalesOf)`,
        );

        if (dropped.length > 0) {
            const preview = dropped
                .slice(0, 8)
                .map((entry) => `${entry.requested} -> ${entry.resolved}`)
                .join(', ');
            console.log(`  dropped examples: ${preview}`);
            if (dropped.length > 8) {
                console.log(`  ...and ${dropped.length - 8} more`);
            }
        }
    }

    console.log('--------------------------------');
    console.log(`Total combos checked: ${totalCombos}`);
    console.log(`Retain requested region: ${retainedRegionCount}/${totalCombos}`);
    console.log(
        `Accepted by supportedLocalesOf: ${acceptedCount}/${totalCombos}`,
    );
}

main();
