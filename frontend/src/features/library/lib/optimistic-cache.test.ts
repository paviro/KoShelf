import { describe, expect, it, vi } from 'vitest';
import { QueryClient } from '@tanstack/react-query';

import type {
    LibraryAnnotation,
    LibraryDetailData,
    LibraryDetailItem,
} from '../../../shared/contracts';
import type { UpdateAnnotationPayload } from '../../../shared/api-client';
import {
    applyAnnotationDeletion,
    applyAnnotationUpdate,
    applyItemUpdate,
    patchDetailCache,
} from './optimistic-cache';

function makeAnnotation(
    id: string,
    overrides?: Partial<LibraryAnnotation>,
): LibraryAnnotation {
    return { id, text: `Text ${id}`, note: null, color: 'yellow', drawer: 'lighten', ...overrides };
}

function makeDetailData(
    overrides?: Partial<LibraryDetailData>,
): LibraryDetailData {
    return {
        item: {
            id: 'item-1',
            title: 'Test Book',
            authors: ['Author'],
            status: 'reading',
            cover_url: '/covers/item-1.webp',
            content_type: 'book',
            format: 'epub',
        } as LibraryDetailItem,
        highlights: [makeAnnotation('h1'), makeAnnotation('h2')],
        bookmarks: [makeAnnotation('b1')],
        statistics: {
            item_stats: { highlights: 2, bookmarks: 1 },
            session_stats: null,
        },
        ...overrides,
    };
}

// ── applyAnnotationUpdate ────────────────────────────────────────────

describe('applyAnnotationUpdate', () => {
    it('updates a matching highlight and leaves others unchanged', () => {
        const data = makeDetailData();
        const payload: UpdateAnnotationPayload = { note: 'new note' };
        const result = applyAnnotationUpdate(data, 'h1', payload);

        expect(result.highlights![0].note).toBe('new note');
        expect(result.highlights![1]).toEqual(data.highlights![1]);
    });

    it('updates a matching bookmark', () => {
        const data = makeDetailData();
        const payload: UpdateAnnotationPayload = { color: 'blue' };
        const result = applyAnnotationUpdate(data, 'b1', payload);

        expect(result.bookmarks![0].color).toBe('blue');
    });

    it('returns identical structure when annotationId matches nothing', () => {
        const data = makeDetailData();
        const result = applyAnnotationUpdate(data, 'nonexistent', { note: 'x' });

        expect(result.highlights).toEqual(data.highlights);
        expect(result.bookmarks).toEqual(data.bookmarks);
    });

    it('handles null highlights list', () => {
        const data = makeDetailData({ highlights: null });
        const result = applyAnnotationUpdate(data, 'h1', { note: 'x' });

        expect(result.highlights).toBeUndefined();
    });

    it('handles undefined bookmarks list', () => {
        const data = makeDetailData({ bookmarks: undefined });
        const result = applyAnnotationUpdate(data, 'b1', { note: 'x' });

        expect(result.bookmarks).toBeUndefined();
    });

    it('does not mutate the original data', () => {
        const data = makeDetailData();
        const originalHighlights = [...data.highlights!];
        applyAnnotationUpdate(data, 'h1', { note: 'changed' });

        expect(data.highlights).toEqual(originalHighlights);
    });
});

// ── applyAnnotationDeletion ──────────────────────────────────────────

describe('applyAnnotationDeletion', () => {
    it('removes a highlight and decrements highlight count', () => {
        const data = makeDetailData();
        const result = applyAnnotationDeletion(data, 'h1');

        expect(result.highlights).toHaveLength(1);
        expect(result.highlights![0].id).toBe('h2');
        expect(result.statistics!.item_stats!.highlights).toBe(1);
    });

    it('removes a bookmark and decrements bookmark count', () => {
        const data = makeDetailData();
        const result = applyAnnotationDeletion(data, 'b1');

        expect(result.bookmarks).toHaveLength(0);
        expect(result.statistics!.item_stats!.bookmarks).toBe(0);
    });

    it('clamps count at 0 when already zero', () => {
        const data = makeDetailData({
            statistics: {
                item_stats: { highlights: 0, bookmarks: 0 },
                session_stats: null,
            },
        });
        const result = applyAnnotationDeletion(data, 'h1');

        expect(result.statistics!.item_stats!.highlights).toBe(0);
    });

    it('handles null highlights count', () => {
        const data = makeDetailData({
            statistics: {
                item_stats: { highlights: null, bookmarks: 1 },
                session_stats: null,
            },
        });
        const result = applyAnnotationDeletion(data, 'h1');

        expect(result.statistics!.item_stats!.highlights).toBe(0);
    });

    it('returns unchanged data when annotationId matches nothing', () => {
        const data = makeDetailData();
        const result = applyAnnotationDeletion(data, 'nonexistent');

        expect(result.highlights).toHaveLength(2);
        expect(result.bookmarks).toHaveLength(1);
        expect(result.statistics!.item_stats!.highlights).toBe(2);
    });

    it('handles missing statistics', () => {
        const data = makeDetailData({ statistics: null });
        const result = applyAnnotationDeletion(data, 'h1');

        expect(result.statistics).toBeNull();
    });

    it('handles missing item_stats within statistics', () => {
        const data = makeDetailData({
            statistics: { item_stats: null, session_stats: null },
        });
        const result = applyAnnotationDeletion(data, 'h1');

        expect(result.statistics!.item_stats).toBeNull();
    });

    it('does not mutate the original data', () => {
        const data = makeDetailData();
        const originalLen = data.highlights!.length;
        applyAnnotationDeletion(data, 'h1');

        expect(data.highlights).toHaveLength(originalLen);
    });
});

// ── applyItemUpdate ──────────────────────────────────────────────────

describe('applyItemUpdate', () => {
    it('sets review_note', () => {
        const data = makeDetailData();
        const result = applyItemUpdate(data, { review_note: 'Great book' });

        expect(result.item.review_note).toBe('Great book');
    });

    it('does not touch review_note when undefined in payload', () => {
        const data = makeDetailData();
        data.item.review_note = 'existing';
        const result = applyItemUpdate(data, { rating: 5 });

        expect(result.item.review_note).toBe('existing');
    });

    it('sets rating to a number', () => {
        const data = makeDetailData();
        const result = applyItemUpdate(data, { rating: 4 });

        expect(result.item.rating).toBe(4);
    });

    it('clears rating to null when payload.rating is 0', () => {
        const data = makeDetailData();
        const result = applyItemUpdate(data, { rating: 0 });

        expect(result.item.rating).toBeNull();
    });

    it('sets status', () => {
        const data = makeDetailData();
        const result = applyItemUpdate(data, { status: 'complete' });

        expect(result.item.status).toBe('complete');
    });

    it('applies multiple fields simultaneously', () => {
        const data = makeDetailData();
        const result = applyItemUpdate(data, {
            review_note: 'note',
            rating: 3,
            status: 'complete',
        });

        expect(result.item.review_note).toBe('note');
        expect(result.item.rating).toBe(3);
        expect(result.item.status).toBe('complete');
    });

    it('does not mutate the original data', () => {
        const data = makeDetailData();
        const originalTitle = data.item.title;
        applyItemUpdate(data, { review_note: 'changed' });

        expect(data.item.review_note).toBeUndefined();
        expect(data.item.title).toBe(originalTitle);
    });
});

// ── patchDetailCache ─────────────────────────────────────────────────

describe('patchDetailCache', () => {
    const queryKey = ['test', 'detail'];

    it('applies updater and returns previous value', () => {
        const qc = new QueryClient();
        const data = makeDetailData();
        qc.setQueryData(queryKey, data);

        const updater = vi.fn((d: LibraryDetailData) => ({
            ...d,
            item: { ...d.item, title: 'Updated' },
        }));
        const previous = patchDetailCache(qc, queryKey, updater);

        expect(previous).toBe(data);
        expect(updater).toHaveBeenCalledWith(data);
        expect(qc.getQueryData<LibraryDetailData>(queryKey)!.item.title).toBe(
            'Updated',
        );
    });

    it('returns undefined when cache is empty', () => {
        const qc = new QueryClient();
        const updater = vi.fn();
        const previous = patchDetailCache(qc, queryKey, updater);

        expect(previous).toBeUndefined();
    });

    it('does not call updater when cache is empty', () => {
        const qc = new QueryClient();
        const updater = vi.fn();
        patchDetailCache(qc, queryKey, updater);

        expect(updater).not.toHaveBeenCalled();
    });
});
