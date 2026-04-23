import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import {
    readAnnotationSortOrder,
    writeAnnotationSortOrder,
} from './useAnnotationSortOrder';

const ROUTE_STATE_KEY = 'koshelf_route_state_books-detail';

beforeEach(() => {
    localStorage.clear();
});

afterEach(() => {
    localStorage.clear();
});

describe('readAnnotationSortOrder', () => {
    it('returns the default when storage is empty', () => {
        expect(
            readAnnotationSortOrder('books-detail', 'highlights', 'asc'),
        ).toBe('asc');
    });

    it('returns the persisted value', () => {
        localStorage.setItem(
            ROUTE_STATE_KEY,
            JSON.stringify({ annotationSort: { highlights: 'desc' } }),
        );
        expect(
            readAnnotationSortOrder('books-detail', 'highlights', 'asc'),
        ).toBe('desc');
    });

    it('coerces malformed persisted values to the default', () => {
        localStorage.setItem(
            ROUTE_STATE_KEY,
            JSON.stringify({ annotationSort: { highlights: 'garbage' } }),
        );
        expect(
            readAnnotationSortOrder('books-detail', 'highlights', 'asc'),
        ).toBe('asc');
    });

    it('ignores non-object persisted fields', () => {
        localStorage.setItem(
            ROUTE_STATE_KEY,
            JSON.stringify({ annotationSort: 'not-an-object' }),
        );
        expect(
            readAnnotationSortOrder('books-detail', 'highlights', 'asc'),
        ).toBe('asc');
    });
});

describe('writeAnnotationSortOrder', () => {
    it('persists the value under the correct route state key', () => {
        writeAnnotationSortOrder('books-detail', 'highlights', 'desc');

        const raw = localStorage.getItem(ROUTE_STATE_KEY);
        expect(raw).not.toBeNull();
        const parsed = JSON.parse(raw ?? '{}');
        expect(parsed.annotationSort).toEqual({ highlights: 'desc' });
    });

    it('keeps sibling section values intact when writing', () => {
        localStorage.setItem(
            ROUTE_STATE_KEY,
            JSON.stringify({
                annotationSort: { bookmarks: 'desc' },
            }),
        );

        writeAnnotationSortOrder('books-detail', 'highlights', 'desc');

        const parsed = JSON.parse(
            localStorage.getItem(ROUTE_STATE_KEY) ?? '{}',
        );
        expect(parsed.annotationSort).toEqual({
            highlights: 'desc',
            bookmarks: 'desc',
        });
    });

    it('is a round-trip with readAnnotationSortOrder', () => {
        writeAnnotationSortOrder('books-detail', 'bookmarks', 'desc');
        expect(
            readAnnotationSortOrder('books-detail', 'bookmarks', 'asc'),
        ).toBe('desc');
        expect(
            readAnnotationSortOrder('books-detail', 'highlights', 'asc'),
        ).toBe('asc');
    });
});
