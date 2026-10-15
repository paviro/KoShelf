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
        availability: 'available',
        title: 'A book',
        authors: ['Author'],
        status: 'reading',
        content_type: 'book',
        cover_url: '/cover.webp',
    };
    const items = [
        base,
        { ...base, id: 'b', availability: 'archived' as const },
        { ...base, id: 'c', title: 'Other', availability: 'archived' as const },
        {
            ...base,
            id: 'd',
            status: 'complete' as const,
            availability: 'archived' as const,
        },
    ];
    it('defaults to all and filters by explicit availability', () => {
        expect(
            items.filter((item) => itemMatchesAvailability(item, 'all')),
        ).toHaveLength(4);
        expect(
            items
                .filter((item) => itemMatchesAvailability(item, 'available'))
                .map((item) => item.id),
        ).toEqual(['a']);
    });
    it('keeps metadata entries in the active library', () => {
        const item: LibraryListItem = { ...base, availability: 'metadata' };
        expect(itemMatchesAvailability(item, 'available')).toBe(true);
        expect(itemMatchesAvailability(item, 'archived')).toBe(false);
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
