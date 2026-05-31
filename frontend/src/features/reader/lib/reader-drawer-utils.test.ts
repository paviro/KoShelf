import { describe, expect, it } from 'vitest';

import type { LibraryAnnotation } from '../../library/api/library-data';
import {
    groupAnnotationsByChapter,
    sortReaderAnnotationsByPage,
} from './reader-drawer-utils';

function makeAnnotation(
    id: string,
    pageno?: number | null,
    chapter?: string | null,
): LibraryAnnotation {
    return {
        id,
        pageno,
        chapter,
    };
}

describe('sortReaderAnnotationsByPage', () => {
    it('sorts reader annotations by page number ascending', () => {
        const annotations = [
            makeAnnotation('page-30', 30),
            makeAnnotation('page-10', 10),
            makeAnnotation('page-20', 20),
        ];

        expect(
            sortReaderAnnotationsByPage(annotations).map(
                (annotation) => annotation.id,
            ),
        ).toEqual(['page-10', 'page-20', 'page-30']);
    });

    it('places annotations without page numbers after page-backed annotations', () => {
        const annotations = [
            makeAnnotation('missing'),
            makeAnnotation('page-20', 20),
            makeAnnotation('page-10', 10),
        ];

        expect(
            sortReaderAnnotationsByPage(annotations).map(
                (annotation) => annotation.id,
            ),
        ).toEqual(['page-10', 'page-20', 'missing']);
    });

    it('preserves original relative order for annotations on the same page', () => {
        const annotations = [
            makeAnnotation('first-page-10', 10),
            makeAnnotation('page-5', 5),
            makeAnnotation('second-page-10', 10),
        ];

        expect(
            sortReaderAnnotationsByPage(annotations).map(
                (annotation) => annotation.id,
            ),
        ).toEqual(['page-5', 'first-page-10', 'second-page-10']);
    });

    it('groups annotations after page sorting', () => {
        const annotations = [
            makeAnnotation('chapter-2', 20, 'Chapter 2'),
            makeAnnotation('chapter-1', 10, 'Chapter 1'),
        ];

        const groups = groupAnnotationsByChapter(
            sortReaderAnnotationsByPage(annotations),
        );

        expect(groups.map((group) => group.chapter)).toEqual([
            'Chapter 1',
            'Chapter 2',
        ]);
        expect(groups.map((group) => group.annotations[0].id)).toEqual([
            'chapter-1',
            'chapter-2',
        ]);
    });
});
