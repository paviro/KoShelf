/**
 * Optimistic updaters for the library detail cache.  Each `apply*` function
 * is a pure, immutable updater that returns a new `LibraryDetailData` with
 * the relevant fields changed.  Used by the mutation hooks in
 * `library-mutations.ts`.
 */

import type { QueryClient } from '@tanstack/react-query';

import type {
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from '../../../shared/api-client';
import type {
    LibraryAnnotation,
    LibraryDetailData,
    LibraryStatus,
} from '../api/library-data';

/**
 * Snapshot the current detail cache, apply `updater`, and return the
 * previous value for rollback.  Returns `undefined` if the cache was empty.
 */
export function patchDetailCache(
    queryClient: QueryClient,
    queryKey: readonly unknown[],
    updater: (data: LibraryDetailData) => LibraryDetailData,
): LibraryDetailData | undefined {
    const previous = queryClient.getQueryData<LibraryDetailData>(queryKey);
    if (previous) {
        queryClient.setQueryData<LibraryDetailData>(queryKey, updater(previous));
    }
    return previous;
}

function updateInList(
    list: LibraryAnnotation[] | null | undefined,
    annotationId: string,
    payload: UpdateAnnotationPayload,
): LibraryAnnotation[] | null | undefined {
    return list?.map((a) =>
        a.id === annotationId ? { ...a, ...payload } : a,
    );
}

/** Patch a single annotation's note / color / drawer in the cache. */
export function applyAnnotationUpdate(
    data: LibraryDetailData,
    annotationId: string,
    payload: UpdateAnnotationPayload,
): LibraryDetailData {
    return {
        ...data,
        highlights: updateInList(data.highlights, annotationId, payload),
        bookmarks: updateInList(data.bookmarks, annotationId, payload),
    };
}

/** Remove an annotation and adjust the stat counters. */
export function applyAnnotationDeletion(
    data: LibraryDetailData,
    annotationId: string,
): LibraryDetailData {
    const wasHighlight = data.highlights?.some((a) => a.id === annotationId);
    const wasBookmark = data.bookmarks?.some((a) => a.id === annotationId);

    const itemStats = data.statistics?.item_stats;

    return {
        ...data,
        highlights: data.highlights?.filter((a) => a.id !== annotationId),
        bookmarks: data.bookmarks?.filter((a) => a.id !== annotationId),
        statistics: data.statistics
            ? {
                  ...data.statistics,
                  item_stats: itemStats
                      ? {
                            ...itemStats,
                            highlights: wasHighlight
                                ? Math.max(0, (itemStats.highlights ?? 0) - 1)
                                : itemStats.highlights,
                            bookmarks: wasBookmark
                                ? Math.max(0, (itemStats.bookmarks ?? 0) - 1)
                                : itemStats.bookmarks,
                        }
                      : itemStats,
              }
            : data.statistics,
    };
}

/** Patch item-level fields (review note, rating, status). */
export function applyItemUpdate(
    data: LibraryDetailData,
    payload: UpdateItemPayload,
): LibraryDetailData {
    const item = { ...data.item };
    if (payload.review_note !== undefined) {
        item.review_note = payload.review_note;
    }
    if (payload.rating !== undefined) {
        item.rating = payload.rating || null;
    }
    if (payload.status !== undefined) {
        item.status = payload.status as LibraryStatus;
    }
    return { ...data, item };
}
