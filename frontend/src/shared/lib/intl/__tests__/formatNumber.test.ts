import { beforeEach, describe, expect, it, vi } from 'vitest';

const { getLanguageMock } = vi.hoisted(() => ({
    getLanguageMock: vi.fn(() => 'en-US'),
}));

vi.mock('../../../i18n', () => ({
    translation: {
        getLanguage: getLanguageMock,
    },
}));

import { formatNumber } from '../formatNumber';

describe('mixed locale number formatting', () => {
    beforeEach(() => {
        getLanguageMock.mockReturnValue('en-US');
    });

    it('uses the selected region separators for mixed locales', () => {
        const options = {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
        } satisfies Intl.NumberFormatOptions;

        expect(formatNumber(1234567.89, options, 'en-HU')).toBe(
            new Intl.NumberFormat('hu-HU', options).format(1234567.89),
        );
    });

    it('keeps official language-region locales unchanged', () => {
        const options = {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
        } satisfies Intl.NumberFormatOptions;

        expect(formatNumber(1234567.89, options, 'fr-CA')).toBe(
            new Intl.NumberFormat('fr-CA', options).format(1234567.89),
        );
    });

    it('uses the active locale by default', () => {
        const options = {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
        } satisfies Intl.NumberFormatOptions;
        getLanguageMock.mockReturnValue('fr-DE');

        expect(formatNumber(1234567.89, options)).toBe(
            new Intl.NumberFormat('de-DE', options).format(1234567.89),
        );
    });
});
