import { describe, expect, it } from 'vitest';

import type { LibraryAnnotation } from '../../library/api/library-data';
import { resolveTargetAnnotation } from './reader-annotation-url';

function annotation(id: string): LibraryAnnotation {
    return { id };
}

describe('resolveTargetAnnotation', () => {
    const highlights = [
        annotation('highlight-first'),
        annotation('highlight-second'),
    ];
    const bookmarks = [
        annotation('bookmark-first'),
        annotation('bookmark-second'),
    ];

    it('resolves highlights by annotation ID', () => {
        expect(
            resolveTargetAnnotation(
                highlights,
                bookmarks,
                'highlight-second',
                null,
            ),
        ).toEqual({
            annotation: highlights[1],
            index: 1,
        });
    });

    it('resolves bookmarks by annotation ID', () => {
        expect(
            resolveTargetAnnotation(
                highlights,
                bookmarks,
                null,
                'bookmark-first',
            ),
        ).toEqual({
            annotation: bookmarks[0],
            index: 0,
        });
    });

    it('gives bookmark params precedence when both params are present', () => {
        expect(
            resolveTargetAnnotation(
                highlights,
                bookmarks,
                'highlight-first',
                'bookmark-second',
            ),
        ).toEqual({
            annotation: bookmarks[1],
            index: 1,
        });
    });

    it('does not treat numeric params as array indexes', () => {
        expect(
            resolveTargetAnnotation(highlights, bookmarks, '1', null),
        ).toBeNull();
    });

    it('returns null for unknown IDs', () => {
        expect(
            resolveTargetAnnotation(highlights, bookmarks, 'missing', null),
        ).toBeNull();
    });

    it('returns null without annotation params', () => {
        expect(
            resolveTargetAnnotation(highlights, bookmarks, null, null),
        ).toBeNull();
    });
});
