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

function resolveAnnotationSectionIndex(
    annotation: LibraryAnnotation,
): number | null {
    if (!annotation.pos0) {
        return null;
    }

    const parsedPosition = parseKoReaderPosition(annotation.pos0);
    return parsedPosition?.spineIndex ?? null;
}

type AnnotationChapterGroup = {
    chapter: string;
    annotations: LibraryAnnotation[];
};

export function groupAnnotationsByChapter(
    annotations: LibraryAnnotation[],
): AnnotationChapterGroup[] {
    const groups: AnnotationChapterGroup[] = [];
    let current: AnnotationChapterGroup | null = null;

    for (const annotation of annotations) {
        const chapter = annotation.chapter ?? '';
        if (!current || current.chapter !== chapter) {
            current = { chapter, annotations: [] };
            groups.push(current);
        }
        current.annotations.push(annotation);
    }

    return groups;
}

export function resolveCurrentGroupIndex(
    groups: AnnotationChapterGroup[],
    normalizedChapter: string,
    currentSectionIndex: number | null,
): number {
    const matchedByChapter = groups.findIndex(
        (group) =>
            group.chapter &&
            normalizeReaderText(group.chapter) === normalizedChapter,
    );
    if (matchedByChapter >= 0) {
        return matchedByChapter;
    }

    const candidateSectionIndexes =
        resolveSectionCandidates(currentSectionIndex);

    for (const sectionIndex of candidateSectionIndexes) {
        const matchedBySection = groups.findIndex((group) =>
            group.annotations.some(
                (annotation) =>
                    resolveAnnotationSectionIndex(annotation) === sectionIndex,
            ),
        );
        if (matchedBySection >= 0) {
            return matchedBySection;
        }
    }

    return -1;
}
