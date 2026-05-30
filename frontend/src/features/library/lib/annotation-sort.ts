import type { LibraryAnnotation } from '../api/library-data';

export type AnnotationSortOrder =
    | 'date-asc'
    | 'date-desc'
    | 'page-asc'
    | 'page-desc';

export type SortedAnnotationEntry = {
    annotation: LibraryAnnotation;
    originalIndex: number;
};

const SORT_ORDER_SEQUENCE: readonly AnnotationSortOrder[] = [
    'page-asc',
    'page-desc',
    'date-asc',
    'date-desc',
];

function annotationTimestamp(annotation: LibraryAnnotation): number | null {
    if (!annotation.datetime) {
        return null;
    }

    const timestamp = Date.parse(annotation.datetime);
    return Number.isNaN(timestamp) ? null : timestamp;
}

function annotationPageNumber(annotation: LibraryAnnotation): number | null {
    return typeof annotation.pageno === 'number' &&
        Number.isFinite(annotation.pageno)
        ? annotation.pageno
        : null;
}

function compareNullableNumber(
    left: number | null,
    right: number | null,
    direction: 'asc' | 'desc',
): number {
    if (left !== null && right !== null) {
        const comparison = direction === 'asc' ? left - right : right - left;
        if (comparison !== 0) {
            return comparison;
        }
    } else if (left !== null) {
        return -1;
    } else if (right !== null) {
        return 1;
    }

    return 0;
}

export function normalizeAnnotationSortOrder(
    value: unknown,
    defaultOrder: AnnotationSortOrder,
): AnnotationSortOrder {
    if (value === 'asc') {
        return 'date-asc';
    }

    if (value === 'desc') {
        return 'date-desc';
    }

    return SORT_ORDER_SEQUENCE.includes(value as AnnotationSortOrder)
        ? (value as AnnotationSortOrder)
        : defaultOrder;
}

export function nextAnnotationSortOrder(
    order: AnnotationSortOrder,
): AnnotationSortOrder {
    const currentIndex = SORT_ORDER_SEQUENCE.indexOf(order);
    const nextIndex =
        currentIndex >= 0 ? (currentIndex + 1) % SORT_ORDER_SEQUENCE.length : 0;
    return SORT_ORDER_SEQUENCE[nextIndex];
}

export function sortedAnnotationEntries(
    annotations: LibraryAnnotation[],
    order: AnnotationSortOrder,
): SortedAnnotationEntry[] {
    return annotations
        .map((annotation, originalIndex) => ({
            annotation,
            originalIndex,
            pageNumber: annotationPageNumber(annotation),
            timestamp: annotationTimestamp(annotation),
        }))
        .sort((left, right) => {
            const comparison = order.startsWith('page-')
                ? compareNullableNumber(
                      left.pageNumber,
                      right.pageNumber,
                      order === 'page-asc' ? 'asc' : 'desc',
                  )
                : compareNullableNumber(
                      left.timestamp,
                      right.timestamp,
                      order === 'date-asc' ? 'asc' : 'desc',
                  );

            if (comparison !== 0) {
                return comparison;
            }

            return left.originalIndex - right.originalIndex;
        })
        .map(({ annotation, originalIndex }) => ({
            annotation,
            originalIndex,
        }));
}
