import { describe, expect, it } from 'vitest';

import type { LibraryAnnotation } from '../api/library-data';
import {
    nextAnnotationSortOrder,
    normalizeAnnotationSortOrder,
    sortedAnnotationEntries,
} from './annotation-sort';

function makeAnnotation(
    id: string,
    datetime?: string | null,
    pageno?: number | null,
): LibraryAnnotation {
    return {
        id,
        datetime,
        pageno,
    };
}

describe('sortedAnnotationEntries', () => {
    it('sorts annotations oldest first in ascending order', () => {
        const annotations = [
            makeAnnotation('newer', '2026-02-01T10:00:00Z'),
            makeAnnotation('older', '2026-01-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'date-asc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['older', 'newer']);
    });

    it('sorts annotations newest first in descending order', () => {
        const annotations = [
            makeAnnotation('older', '2026-01-01T10:00:00Z'),
            makeAnnotation('newer', '2026-02-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'date-desc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['newer', 'older']);
    });

    it('uses the original index as the tie breaker for matching dates', () => {
        const annotations = [
            makeAnnotation('first', '2026-01-01T10:00:00Z'),
            makeAnnotation('second', '2026-01-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'date-desc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['first', 'second']);
    });

    it('places missing and invalid dates after valid dates', () => {
        const annotations = [
            makeAnnotation('missing', null),
            makeAnnotation('valid', '2026-01-01T10:00:00Z'),
            makeAnnotation('invalid', 'not-a-date'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'date-asc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['valid', 'missing', 'invalid']);
    });

    it('sorts annotations by page number ascending', () => {
        const annotations = [
            makeAnnotation('page-30', null, 30),
            makeAnnotation('page-10', null, 10),
            makeAnnotation('page-20', null, 20),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'page-asc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['page-10', 'page-20', 'page-30']);
    });

    it('sorts annotations by page number descending', () => {
        const annotations = [
            makeAnnotation('page-10', null, 10),
            makeAnnotation('page-30', null, 30),
            makeAnnotation('page-20', null, 20),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'page-desc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['page-30', 'page-20', 'page-10']);
    });

    it('places missing page numbers after valid page numbers', () => {
        const annotations = [
            makeAnnotation('missing'),
            makeAnnotation('page-10', null, 10),
            makeAnnotation('page-20', null, 20),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'page-desc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['page-20', 'page-10', 'missing']);
    });

    it('preserves original indexes as sort metadata after sorting', () => {
        const annotations = [
            makeAnnotation('middle', '2026-02-01T10:00:00Z'),
            makeAnnotation('newest', '2026-03-01T10:00:00Z'),
            makeAnnotation('oldest', '2026-01-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'date-asc').map((entry) => [
                entry.annotation.id,
                entry.originalIndex,
            ]),
        ).toEqual([
            ['oldest', 2],
            ['middle', 0],
            ['newest', 1],
        ]);
    });
});

describe('normalizeAnnotationSortOrder', () => {
    it('maps legacy persisted date values to explicit date sort states', () => {
        expect(normalizeAnnotationSortOrder('asc', 'date-desc')).toBe(
            'date-asc',
        );
        expect(normalizeAnnotationSortOrder('desc', 'date-asc')).toBe(
            'date-desc',
        );
    });

    it('accepts current persisted sort states', () => {
        expect(normalizeAnnotationSortOrder('page-asc', 'date-asc')).toBe(
            'page-asc',
        );
        expect(normalizeAnnotationSortOrder('page-desc', 'date-asc')).toBe(
            'page-desc',
        );
    });

    it('returns the default for unknown sort states', () => {
        expect(normalizeAnnotationSortOrder('unknown', 'date-asc')).toBe(
            'date-asc',
        );
    });
});

describe('nextAnnotationSortOrder', () => {
    it('cycles through page and date sort states', () => {
        expect(nextAnnotationSortOrder('page-asc')).toBe('page-desc');
        expect(nextAnnotationSortOrder('page-desc')).toBe('date-asc');
        expect(nextAnnotationSortOrder('date-asc')).toBe('date-desc');
        expect(nextAnnotationSortOrder('date-desc')).toBe('page-asc');
    });
});
