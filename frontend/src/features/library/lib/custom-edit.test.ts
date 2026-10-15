import { describe, expect, it } from 'vitest';

import {
    buildMetadataPatch,
    isMetadataOverride,
    isValidSeriesIndex,
} from './custom-edit';

const edit = {
    base: { title: 'Original', authors: 'A\nB' },
    overrides: { title: 'Custom' } as Record<string, string>,
};

describe('isMetadataOverride', () => {
    it('ignores whitespace and empty list entries', () => {
        expect(isMetadataOverride('title', ' Original ', 'Original')).toBe(
            false,
        );
        expect(isMetadataOverride('authors', 'A\n\n B \n', 'A\nB')).toBe(false);
        expect(isMetadataOverride('authors', 'A\nB\nC', 'A\nB')).toBe(true);
    });

    it('treats blank text as no override', () => {
        expect(isMetadataOverride('title', '  ', 'Original')).toBe(false);
    });
});

describe('buildMetadataPatch', () => {
    it('clears a stored override when the text matches the original', () => {
        expect(buildMetadataPatch(edit, { title: 'Original' })).toEqual({
            title: null,
        });
    });

    it('omits fields whose text matches the original without an override', () => {
        expect(buildMetadataPatch(edit, { authors: 'A\nB\n' })).toEqual({});
    });

    it('omits unchanged overrides and sends new values', () => {
        expect(
            buildMetadataPatch(edit, { title: 'Custom', authors: 'C' }),
        ).toEqual({ authors: 'C' });
    });
});

describe('series index', () => {
    it('accepts numbers and blanks only', () => {
        for (const value of ['2', ' 2.5 ', '.5', '-1', '1e2', '']) {
            expect(isValidSeriesIndex(value)).toBe(true);
        }
        for (const value of ['one', '2a', '0x10', 'Infinity', '1,5']) {
            expect(isValidSeriesIndex(value)).toBe(false);
        }
    });

    it('compares numerically', () => {
        expect(isMetadataOverride('series_index', '1', '1.0')).toBe(false);
        expect(isMetadataOverride('series_index', '1.5', '1')).toBe(true);
    });
});
