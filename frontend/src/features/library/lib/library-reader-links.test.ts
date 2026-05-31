import { describe, expect, it } from 'vitest';

import { annotationReaderHref } from './library-reader-links';

describe('annotationReaderHref', () => {
    it('builds highlight links with the annotation ID', () => {
        expect(
            annotationReaderHref('/books/abc/read', 'highlight', 'ann-123'),
        ).toBe('/books/abc/read?highlight=ann-123');
    });

    it('builds bookmark links with the annotation ID', () => {
        expect(
            annotationReaderHref('/books/abc/read', 'bookmark', 'ann-456'),
        ).toBe('/books/abc/read?bookmark=ann-456');
    });

    it('encodes annotation IDs for query strings', () => {
        expect(
            annotationReaderHref('/books/abc/read', 'highlight', 'ann 123'),
        ).toBe('/books/abc/read?highlight=ann+123');
    });

    it('returns undefined without a reader base href', () => {
        expect(
            annotationReaderHref(null, 'highlight', 'ann-123'),
        ).toBeUndefined();
    });
});
