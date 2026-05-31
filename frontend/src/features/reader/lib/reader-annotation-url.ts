import type { LibraryAnnotation } from '../../library/api/library-data';

export function resolveTargetAnnotation(
    highlights: LibraryAnnotation[],
    bookmarks: LibraryAnnotation[],
    highlightId: string | null,
    bookmarkId: string | null,
): { annotation: LibraryAnnotation; index: number } | null {
    const targetId = bookmarkId !== null ? bookmarkId : highlightId;
    const annotations = bookmarkId !== null ? bookmarks : highlights;

    if (targetId === null) {
        return null;
    }

    const index = annotations.findIndex(
        (annotation) => annotation.id === targetId,
    );

    if (index === -1) {
        return null;
    }

    return { annotation: annotations[index], index };
}
