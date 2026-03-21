import { describe, expect, it } from 'vitest';

import { resolveReaderFontOverride } from './reader-fonts';

function expectFontStackContains(stack: string, family: string): void {
    expect(stack).toContain(`'${family}'`);
}

describe('resolveReaderFontOverride', () => {
    it('falls back to Noto Serif when font face is missing', async () => {
        const resolved = await resolveReaderFontOverride(null);

        expect(resolved.requestedFamily).toBe('Noto Serif');
        expect(resolved.fallbackFamily).toBe('Noto Serif');
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Serif');
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans');
        expect(typeof resolved.fontFaceCss).toBe('string');
        expect(resolved.fontFamilyCssValue.endsWith(', serif')).toBe(true);
    });

    it('keeps the requested KOReader font and appends Noto Serif fallback', async () => {
        const resolved = await resolveReaderFontOverride(
            '"Minion Pro Cond", serif embedded',
        );

        expect(resolved.requestedFamily).toBe('Minion Pro Cond');
        expect(resolved.fallbackFamily).toBe('Noto Serif');
        expect(
            resolved.fontFamilyCssValue.startsWith("'Minion Pro Cond'"),
        ).toBe(true);
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Serif');
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans');
    });

    it('treats unknown fonts as unpackaged and uses full fallback chain', async () => {
        const resolved = await resolveReaderFontOverride('Some Custom Font');

        expect(resolved.requestedFamily).toBe('Some Custom Font');
        expect(resolved.fallbackFamily).toBe('Noto Serif');
        expect(
            resolved.fontFamilyCssValue.startsWith("'Some Custom Font'"),
        ).toBe(true);
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Serif');
    });
});
