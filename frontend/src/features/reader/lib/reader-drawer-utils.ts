import type { LibraryAnnotation } from '../../library/api/library-data';
import { parseKoReaderPosition } from './reader-position-parser';

export function normalizeReaderText(value: string): string {
    return value.trim().replace(/\s+/g, ' ').toLowerCase();
}

export function resolveSectionCandidates(
    currentSectionIndex: number | null,
): number[] {
    if (
        currentSectionIndex === null ||
        Number.isNaN(currentSectionIndex) ||
        currentSectionIndex < 0
    ) {
        return [];
    }

    return currentSectionIndex > 0
        ? [currentSectionIndex, currentSectionIndex - 1]
        : [currentSectionIndex];
}

export function resolveAnnotationSectionIndex(
    annotation: LibraryAnnotation,
): number | null {
    if (!annotation.pos0) {
        return null;
    }

    const parsedPosition = parseKoReaderPosition(annotation.pos0);
    return parsedPosition?.spineIndex ?? null;
}
