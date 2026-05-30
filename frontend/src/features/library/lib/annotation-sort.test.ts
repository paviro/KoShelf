import { describe, expect, it } from 'vitest';

import type { LibraryAnnotation } from '../api/library-data';
import { sortedAnnotationEntries } from './annotation-sort';

function makeAnnotation(
    id: string,
    datetime?: string | null,
): LibraryAnnotation {
    return {
        id,
        datetime,
    };
}

describe('sortedAnnotationEntries', () => {
    it('sorts annotations oldest first in ascending order', () => {
        const annotations = [
            makeAnnotation('newer', '2026-02-01T10:00:00Z'),
            makeAnnotation('older', '2026-01-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'asc').map(
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
            sortedAnnotationEntries(annotations, 'desc').map(
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
            sortedAnnotationEntries(annotations, 'desc').map(
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
            sortedAnnotationEntries(annotations, 'asc').map(
                (entry) => entry.annotation.id,
            ),
        ).toEqual(['valid', 'missing', 'invalid']);
    });

    it('preserves original indexes for reader links after sorting', () => {
        const annotations = [
            makeAnnotation('middle', '2026-02-01T10:00:00Z'),
            makeAnnotation('newest', '2026-03-01T10:00:00Z'),
            makeAnnotation('oldest', '2026-01-01T10:00:00Z'),
        ];

        expect(
            sortedAnnotationEntries(annotations, 'asc').map((entry) => [
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
