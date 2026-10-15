import type { LibraryAvailability } from './contracts';

export function documentAvailable(
    item: { availability?: LibraryAvailability } | null | undefined,
): boolean {
    return item?.availability === 'available';
}
