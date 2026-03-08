import { beforeEach, describe, expect, it, vi } from 'vitest';

const { getLanguageMock } = vi.hoisted(() => ({
    getLanguageMock: vi.fn(() => 'en-US'),
}));

vi.mock('../../../i18n', () => ({
    translation: {
        getLanguage: getLanguageMock,
    },
}));

import {
    formatDateObject,
    formatDateObjectToParts,
    formatPlainDate,
    formatPlainDateRange,
} from '../formatDate';
import { getRegionPatternLocale } from '../locale-options';

const SAMPLE_DATE = new Date(Date.UTC(2025, 11, 27, 12, 34, 0, 0));
const MARCH_DATE = new Date(Date.UTC(2026, 2, 5, 19, 20, 0, 0));

describe('mixed locale date formatting', () => {
    beforeEach(() => {
        getLanguageMock.mockReturnValue('en-US');
    });

    it('derives region-native pattern locales generically', () => {
        expect(getRegionPatternLocale('HU', 'en')).toBe('hu-HU');
        expect(getRegionPatternLocale('DE', 'fr')).toBe('de-DE');
        expect(getRegionPatternLocale('CA', 'fr')).toBe('fr-CA');
    });

    it('uses the selected region ordering for plain dates', () => {
        getLanguageMock.mockReturnValue('en-HU');

        expect(
            formatPlainDate('2025-12-27', {
                monthStyle: 'short',
                yearDisplay: 'always',
            }),
        ).toBe('2025. Dec 27.');
    });

    it('applies the same rule across other mixed locale pairs', () => {
        getLanguageMock.mockReturnValue('fr-DE');

        expect(
            formatPlainDate('2025-12-27', {
                monthStyle: 'short',
                yearDisplay: 'always',
            }),
        ).toBe('27. déc. 2025');
    });

    it('formats mixed-locale ranges deterministically', () => {
        getLanguageMock.mockReturnValue('en-HU');

        expect(
            formatPlainDateRange('2025-12-27', '2025-12-31', {
                monthStyle: 'short',
                yearDisplay: 'always',
            }),
        ).toBe('2025. Dec 27. – 2025. Dec 31.');
    });

    it('uses neutral separators when region patterns contain words', () => {
        const formatted = formatDateObject(
            MARCH_DATE,
            {
                dateStyle: 'full',
                timeStyle: 'short',
                timeZone: 'UTC',
            },
            '--',
            'de-IT',
        );

        expect(formatted).toBe('Donnerstag 5 März 2026, 19:20');
        expect(formatted.includes('alle ore')).toBe(false);
    });

    it('removes connector words from mixed plain dates and ranges', () => {
        getLanguageMock.mockReturnValue('de-ES');

        expect(
            formatPlainDate('2026-03-05', {
                monthStyle: 'long',
                yearDisplay: 'always',
            }),
        ).toBe('5 März 2026');
        expect(
            formatPlainDateRange('2026-03-05', '2026-03-07', {
                monthStyle: 'long',
                yearDisplay: 'always',
            }),
        ).toBe('5 März 2026 – 7 März 2026');
    });

    it('keeps official region-language locales on their native pattern', () => {
        expect(
            formatDateObject(
                SAMPLE_DATE,
                {
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric',
                    timeZone: 'UTC',
                },
                '--',
                'fr-CA',
            ),
        ).toBe(
            new Intl.DateTimeFormat('fr-CA', {
                year: 'numeric',
                month: 'long',
                day: 'numeric',
                timeZone: 'UTC',
            }).format(SAMPLE_DATE),
        );
    });

    it('drops suffix-style region literals from mixed full date-times', () => {
        expect(
            formatDateObject(
                MARCH_DATE,
                {
                    dateStyle: 'full',
                    timeStyle: 'short',
                    timeZone: 'UTC',
                },
                '--',
                'pt-UA',
            ),
        ).toBe('quinta-feira, 5 março 2026, 19:20');
    });

    it('returns mixed-locale parts in region order with selected-language labels', () => {
        const parts = formatDateObjectToParts(
            SAMPLE_DATE,
            {
                month: 'long',
                year: 'numeric',
                timeZone: 'UTC',
            },
            'en-HU',
        );

        expect(parts).toEqual([
            { type: 'year', value: '2025' },
            { type: 'literal', value: '. ' },
            { type: 'month', value: 'December' },
        ]);
    });

    it('returns month-year parts without foreign glue words', () => {
        const parts = formatDateObjectToParts(
            MARCH_DATE,
            {
                month: 'long',
                year: 'numeric',
                timeZone: 'UTC',
            },
            'de-ES',
        );

        expect(parts).toEqual([
            { type: 'month', value: 'März' },
            { type: 'literal', value: ' ' },
            { type: 'year', value: '2026' },
        ]);
    });
});
