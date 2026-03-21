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
        expectFontStackContains(
            resolved.fontFamilyCssValue,
            'Noto Sans Arabic',
        );
        expectFontStackContains(
            resolved.fontFamilyCssValue,
            'Noto Sans Hebrew',
        );
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans Thai');
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans JP');
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans KR');
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
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Sans JP');
    });

    it('normalizes known aliases to packaged fonts', async () => {
        const resolved = await resolveReaderFontOverride('Noto Sans Arabic UI');

        expect(resolved.requestedFamily).toBe('Noto Sans Arabic');
        expect(resolved.fallbackFamily).toBe('Noto Serif');
        expect(
            resolved.fontFamilyCssValue.startsWith("'Noto Sans Arabic'"),
        ).toBe(true);
        expectFontStackContains(resolved.fontFamilyCssValue, 'Noto Serif');
    });

    it('maps CJK aliases to bundled families', async () => {
        const resolved = await resolveReaderFontOverride('Noto Sans CJKjp');

        expect(resolved.requestedFamily).toBe('Noto Sans JP');
        expect(resolved.fontFamilyCssValue.startsWith("'Noto Sans JP'")).toBe(
            true,
        );
    });
});
