import { StorageManager } from '../../../shared/storage-manager';
import type { LibraryAnnotation } from '../api/library-data';
import type { LibraryCollection } from './library-model';

export const LIBRARY_DETAIL_SECTION_KEYS = [
    'book-overview',
    'reading-stats',
    'review',
    'highlights',
    'bookmarks',
    'additional-info',
] as const;

export type LibraryDetailSectionKey = (typeof LIBRARY_DETAIL_SECTION_KEYS)[number];

export type LibraryDetailSectionVisibilityState = Record<LibraryDetailSectionKey, boolean>;

const DEFAULT_DETAIL_SECTION_STATE: LibraryDetailSectionVisibilityState = {
    'book-overview': true,
    'reading-stats': false,
    review: true,
    highlights: true,
    bookmarks: true,
    'additional-info': false,
};

export function defaultLibraryDetailSectionState(): LibraryDetailSectionVisibilityState {
    return { ...DEFAULT_DETAIL_SECTION_STATE };
}

export function libraryDetailSectionStorageKey(
    collection: LibraryCollection,
):
    | typeof StorageManager.KEYS.ITEM_DETAIL_BOOKS_SECTIONS
    | typeof StorageManager.KEYS.ITEM_DETAIL_COMICS_SECTIONS {
    if (collection === 'comics') {
        return StorageManager.KEYS.ITEM_DETAIL_COMICS_SECTIONS;
    }

    return StorageManager.KEYS.ITEM_DETAIL_BOOKS_SECTIONS;
}

function annotationFingerprint(annotation: LibraryAnnotation): string {
    return [
        annotation.chapter ?? '',
        annotation.datetime ?? '',
        annotation.pageno ?? '',
        annotation.text ?? '',
        annotation.note ?? '',
    ].join('|');
}

function hasHighlightPosition(annotation: LibraryAnnotation): boolean {
    return Boolean(annotation.pos0 || annotation.pos1);
}

export function splitLibraryAnnotations(
    annotations: LibraryAnnotation[],
    bookmarks: LibraryAnnotation[],
): { highlights: LibraryAnnotation[]; bookmarks: LibraryAnnotation[] } {
    if (annotations.some((annotation) => annotation.pos0 || annotation.pos1)) {
        const highlights = annotations.filter((annotation) => hasHighlightPosition(annotation));
        const bookmarkAnnotations = annotations.filter(
            (annotation) => !hasHighlightPosition(annotation),
        );

        return {
            highlights,
            bookmarks: bookmarkAnnotations,
        };
    }

    const bookmarkCounts = new Map<string, number>();

    bookmarks.forEach((bookmark) => {
        const key = annotationFingerprint(bookmark);
        bookmarkCounts.set(key, (bookmarkCounts.get(key) ?? 0) + 1);
    });

    const highlights: LibraryAnnotation[] = [];

    annotations.forEach((annotation) => {
        const key = annotationFingerprint(annotation);
        const remainingCount = bookmarkCounts.get(key) ?? 0;

        if (remainingCount > 0) {
            bookmarkCounts.set(key, remainingCount - 1);
            return;
        }

        highlights.push(annotation);
    });

    return {
        highlights,
        bookmarks,
    };
}
