import type { LibraryAnnotation } from '../../library/api/library-data';
import type {
    ReaderNavigationTarget,
    ReaderTargetingView,
    SectionDocumentCache,
} from '../model/reader-model';
import { resolveCfiFromKoReaderPositions } from './reader-cfi-resolution';
import { parseKoReaderPosition } from './reader-position-parser';
import {
    resolveCfiByTextAcrossSections,
    resolveCfiByTextInSection,
} from './reader-text-search';

function normalizeText(value: string): string {
    return value.trim().replace(/\s+/g, ' ').toLowerCase();
}

export function resolveChapterHref(
    toc: { href: string; label: string }[],
    chapter: string,
): string | null {
    const normalizedChapter = normalizeText(chapter);
    if (!normalizedChapter) {
        return null;
    }

    const exact = toc.find(
        (entry) => normalizeText(entry.label) === normalizedChapter,
    );
    if (exact?.href) {
        return exact.href;
    }

    const withoutPrefix = normalizedChapter.replace(
        /^chapter\s+\d+\s*[:.-]?\s*/,
        '',
    );
    if (!withoutPrefix) {
        return null;
    }

    const fuzzy = toc.find((entry) => {
        const normalizedLabel = normalizeText(entry.label);
        return Boolean(
            normalizedLabel &&
            (normalizedLabel.includes(withoutPrefix) ||
                withoutPrefix.includes(normalizedLabel)),
        );
    });

    return fuzzy?.href ?? null;
}

export function resolvePageHref(
    pageList: { href: string; label: string }[],
    pageno: number,
): string | null {
    const expectedLabel = String(pageno);
    const match = pageList.find((entry) => entry.label === expectedLabel);
    return match?.href ?? null;
}

export async function resolveAnnotationCfi(
    view: ReaderTargetingView,
    annotation: LibraryAnnotation,
    cache?: SectionDocumentCache,
): Promise<string | null> {
    if (annotation.pos0) {
        const byPositions = await resolveCfiFromKoReaderPositions(
            view,
            annotation.pos0,
            annotation.pos1,
            cache,
        );
        if (byPositions) {
            return byPositions;
        }
    }

    if (annotation.text) {
        const parsedPos = annotation.pos0
            ? parseKoReaderPosition(annotation.pos0)
            : null;
        if (parsedPos) {
            const bySectionText = await resolveCfiByTextInSection(
                view,
                annotation.text,
                parsedPos.spineIndex,
                cache,
            );
            if (bySectionText) {
                return bySectionText;
            }
        }

        return resolveCfiByTextAcrossSections(view, annotation.text, cache);
    }

    return null;
}

export async function resolveAnnotationTarget(
    view: ReaderTargetingView,
    annotation: LibraryAnnotation,
): Promise<ReaderNavigationTarget | null> {
    const cfi = await resolveAnnotationCfi(view, annotation);
    if (cfi) {
        return cfi;
    }

    if (annotation.chapter && view.book?.toc?.length) {
        const href = resolveChapterHref(view.book.toc, annotation.chapter);
        if (href) {
            return href;
        }
    }

    if (typeof annotation.pageno === 'number' && view.book?.pageList?.length) {
        const href = resolvePageHref(view.book.pageList, annotation.pageno);
        if (href) {
            return href;
        }
    }

    if (annotation.pos0) {
        const parsed = parseKoReaderPosition(annotation.pos0);
        if (parsed) {
            return parsed.spineIndex;
        }
    }

    return null;
}
