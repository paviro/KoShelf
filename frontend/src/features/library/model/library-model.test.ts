import { describe, expect, it } from 'vitest';
import type { LibraryListItem } from '../api/library-data';
import {
    bucketLibraryItems,
    itemMatchesAvailability,
    itemMatchesSearch,
    sectionMatchesFilter,
} from './library-model';

describe('archive availability filtering', () => {
    const base: LibraryListItem = {
        id: 'a',
        title: 'A book',
        authors: ['Author'],
        status: 'reading',
        content_type: 'book',
        cover_url: '/cover.webp',
    };
    const items = [
        base,
        { ...base, id: 'b', is_archived: true },
        { ...base, id: 'c', title: 'Other', is_archived: true },
        { ...base, id: 'd', status: 'complete' as const, is_archived: true },
    ];
    it('defaults to all and treats older export records as available', () => {
        expect(
            items.filter((item) => itemMatchesAvailability(item, 'all')),
        ).toHaveLength(4);
        expect(
            items
                .filter((item) => itemMatchesAvailability(item, 'available'))
                .map((item) => item.id),
        ).toEqual(['a']);
    });
    it('combines archive selection with search and reading status', () => {
        const buckets = bucketLibraryItems(items);
        const reading = sectionMatchesFilter('reading', 'reading')
            ? buckets.reading
            : [];
        expect(
            reading
                .filter(
                    (item) =>
                        itemMatchesSearch(item, 'a book') &&
                        itemMatchesAvailability(item, 'archived'),
                )
                .map((item) => item.id),
        ).toEqual(['b']);
    });
});
