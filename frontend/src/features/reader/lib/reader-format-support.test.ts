import { describe, expect, it } from 'vitest';

import {
    isReaderFormatSupported,
    normalizeLibraryFormat,
} from './reader-format-support';

describe('normalizeLibraryFormat', () => {
    it('normalizes case and trims whitespace', () => {
        expect(normalizeLibraryFormat('  EPUB  ')).toBe('epub');
    });

    it('removes a leading dot from extensions', () => {
        expect(normalizeLibraryFormat('.cbz')).toBe('cbz');
    });

    it('returns null for empty values', () => {
        expect(normalizeLibraryFormat('   ')).toBeNull();
        expect(normalizeLibraryFormat(null)).toBeNull();
        expect(normalizeLibraryFormat(undefined)).toBeNull();
    });
});

describe('isReaderFormatSupported', () => {
    it('accepts supported formats', () => {
        expect(isReaderFormatSupported('epub')).toBe(true);
        expect(isReaderFormatSupported('fb2')).toBe(true);
        expect(isReaderFormatSupported('mobi')).toBe(true);
        expect(isReaderFormatSupported('cbz')).toBe(true);
    });

    it('rejects unsupported formats', () => {
        expect(isReaderFormatSupported('cbr')).toBe(false);
        expect(isReaderFormatSupported('pdf')).toBe(false);
        expect(isReaderFormatSupported('')).toBe(false);
    });
});
