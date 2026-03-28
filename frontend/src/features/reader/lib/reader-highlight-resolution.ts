import type { LibraryAnnotation } from '../../library/api/library-data';
import type {
    ReaderHighlightValue,
    ReaderTargetingView,
    ResolveHighlightsBySectionOptions,
    SectionDocumentCache,
} from '../model/reader-model';
import {
    resolveCfiFromKoReaderPositionsInDocument,
    sectionDocumentAt,
} from './reader-cfi-resolution';
import { runWithConcurrency } from './reader-concurrency';
import { highlightColor } from './reader-highlight-colors';
import { resolveAnnotationCfi } from './reader-navigation-resolution';
import { parseKoReaderPosition } from './reader-position-parser';
import { resolveCfiByTextInDocument } from './reader-text-search';

function resolveSectionIndexForHighlight(
    view: ReaderTargetingView,
    cfi: string,
    annotation: LibraryAnnotation,
): number {
    let resolvedNavigation:
        | ReturnType<ReaderTargetingView['resolveNavigation']>
        | undefined;

    try {
        resolvedNavigation = view.resolveNavigation(cfi);
    } catch {
        resolvedNavigation = undefined;
    }

    if (resolvedNavigation && typeof resolvedNavigation.index === 'number') {
        return resolvedNavigation.index;
    }

    if (annotation.pos0) {
        const parsed = parseKoReaderPosition(annotation.pos0);
        if (parsed) {
            return parsed.spineIndex;
        }
    }

    return -1;
}

type HighlightValueOptions = {
    color?: string;
    drawer?: string | null;
    note?: string | null;
    target?: boolean;
    annotationId?: string;
};

function buildHighlightValue(
    value: string,
    opts?: HighlightValueOptions,
): ReaderHighlightValue {
    const highlight: ReaderHighlightValue = { value };
    if (opts?.color) highlight.color = opts.color;
    if (opts?.drawer) highlight.drawer = opts.drawer;
    if (opts?.note) highlight.note = opts.note;
    if (opts?.target) highlight.target = true;
    if (opts?.annotationId) highlight.annotationId = opts.annotationId;
    return highlight;
}

function appendHighlightValue(
    bySection: Map<number, ReaderHighlightValue[]>,
    sectionIndex: number,
    value: string,
    opts?: HighlightValueOptions,
) {
    const highlight = buildHighlightValue(value, opts);
    const existing = bySection.get(sectionIndex);
    if (existing) {
        existing.push(highlight);
    } else {
        bySection.set(sectionIndex, [highlight]);
    }
}

export async function resolveHighlightsBySection(
    view: ReaderTargetingView,
    highlights: LibraryAnnotation[],
    isDark: boolean,
    options: ResolveHighlightsBySectionOptions = {},
): Promise<Map<number, ReaderHighlightValue[]>> {
    const bySection = new Map<number, ReaderHighlightValue[]>();
    const sectionDocumentCache: SectionDocumentCache = new Map();

    const hintedBySection = new Map<number, LibraryAnnotation[]>();
    const unhinted: LibraryAnnotation[] = [];

    for (let i = 0; i < highlights.length; i += 1) {
        const annotation = highlights[i];
        if (!annotation.pos0) {
            unhinted.push(annotation);
            continue;
        }

        const parsed = parseKoReaderPosition(annotation.pos0);
        if (!parsed) {
            unhinted.push(annotation);
            continue;
        }

        const existing = hintedBySection.get(parsed.spineIndex);
        if (existing) {
            existing.push(annotation);
        } else {
            hintedBySection.set(parsed.spineIndex, [annotation]);
        }
    }

    const sectionIndexes = Array.from(hintedBySection.keys());
    const orderedSectionIndexes: number[] = [];
    const seenSectionIndexes = new Set<number>();

    for (const sectionIndex of options.prioritizeSectionIndexes ?? []) {
        if (
            !seenSectionIndexes.has(sectionIndex) &&
            hintedBySection.has(sectionIndex)
        ) {
            seenSectionIndexes.add(sectionIndex);
            orderedSectionIndexes.push(sectionIndex);
        }
    }

    for (let i = 0; i < sectionIndexes.length; i += 1) {
        const sectionIndex = sectionIndexes[i];
        if (!seenSectionIndexes.has(sectionIndex)) {
            seenSectionIndexes.add(sectionIndex);
            orderedSectionIndexes.push(sectionIndex);
        }
    }

    const unresolvedHinted: LibraryAnnotation[] = [];

    await runWithConcurrency(
        orderedSectionIndexes,
        options.maxConcurrentSections ?? 4,
        async (sectionIndex) => {
            const sectionAnnotations = hintedBySection.get(sectionIndex);
            if (!sectionAnnotations || sectionAnnotations.length === 0) {
                return;
            }

            const doc = await sectionDocumentAt(
                view,
                sectionIndex,
                sectionDocumentCache,
            );
            const sectionHighlights: ReaderHighlightValue[] = [];

            for (let i = 0; i < sectionAnnotations.length; i += 1) {
                const annotation = sectionAnnotations[i];
                let cfi: string | null = null;

                if (annotation.pos0 && doc) {
                    cfi = resolveCfiFromKoReaderPositionsInDocument(
                        view,
                        annotation.pos0,
                        annotation.pos1,
                        sectionIndex,
                        doc,
                    );
                }

                if (!cfi && annotation.text && doc) {
                    cfi = resolveCfiByTextInDocument(
                        view,
                        annotation.text,
                        sectionIndex,
                        doc,
                    );
                }

                if (cfi) {
                    const color = highlightColor(annotation.color, isDark);
                    const isTarget =
                        annotation.id === options.targetAnnotationId;
                    sectionHighlights.push(
                        buildHighlightValue(cfi, {
                            color,
                            drawer: annotation.drawer,
                            note: annotation.note,
                            target: isTarget,
                            annotationId: annotation.id,
                        }),
                    );
                } else {
                    unresolvedHinted.push(annotation);
                }
            }

            if (sectionHighlights.length > 0) {
                bySection.set(sectionIndex, sectionHighlights);
            }

            if (options.onSectionResolved) {
                await options.onSectionResolved(
                    sectionIndex,
                    sectionHighlights,
                );
            }
        },
    );

    const fallbackAnnotations = [...unhinted, ...unresolvedHinted];

    await runWithConcurrency(fallbackAnnotations, 2, async (annotation) => {
        const cfi = await resolveAnnotationCfi(
            view,
            annotation,
            sectionDocumentCache,
        );
        if (!cfi) {
            return;
        }

        const sectionIndex = resolveSectionIndexForHighlight(
            view,
            cfi,
            annotation,
        );
        if (sectionIndex < 0) {
            return;
        }

        const color = highlightColor(annotation.color, isDark);
        const isTarget = annotation.id === options.targetAnnotationId;
        appendHighlightValue(bySection, sectionIndex, cfi, {
            color,
            drawer: annotation.drawer,
            note: annotation.note,
            target: isTarget,
            annotationId: annotation.id,
        });
    });

    return bySection;
}
