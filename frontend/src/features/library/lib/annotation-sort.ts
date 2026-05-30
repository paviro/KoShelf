import type { LibraryAnnotation } from '../api/library-data';

export type AnnotationSortOrder = 'asc' | 'desc';

export type SortedAnnotationEntry = {
    annotation: LibraryAnnotation;
    originalIndex: number;
};

function annotationTimestamp(annotation: LibraryAnnotation): number | null {
    if (!annotation.datetime) {
        return null;
    }

    const timestamp = Date.parse(annotation.datetime);
    return Number.isNaN(timestamp) ? null : timestamp;
}

export function sortedAnnotationEntries(
    annotations: LibraryAnnotation[],
    order: AnnotationSortOrder,
): SortedAnnotationEntry[] {
    return annotations
        .map((annotation, originalIndex) => ({
            annotation,
            originalIndex,
            timestamp: annotationTimestamp(annotation),
        }))
        .sort((left, right) => {
            if (left.timestamp !== null && right.timestamp !== null) {
                const dateComparison =
                    order === 'asc'
                        ? left.timestamp - right.timestamp
                        : right.timestamp - left.timestamp;
                if (dateComparison !== 0) {
                    return dateComparison;
                }
            } else if (left.timestamp !== null) {
                return -1;
            } else if (right.timestamp !== null) {
                return 1;
            }

            return left.originalIndex - right.originalIndex;
        })
        .map(({ annotation, originalIndex }) => ({
            annotation,
            originalIndex,
        }));
}
