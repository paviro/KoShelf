import type {
    KoReaderPosition,
    ReaderTargetingView,
    SectionDocumentCache,
} from '../model/reader-model';
import {
    clampTextOffset,
    parseKoReaderPosition,
    resolveTextNodeForPath,
} from './reader-position-parser';

export async function sectionDocumentAt(
    view: ReaderTargetingView,
    index: number,
    cache?: SectionDocumentCache,
): Promise<Document | null> {
    const cached = cache?.get(index);
    if (cached) {
        return cached;
    }

    const load = (async () => {
        const sections = view.book?.sections;
        if (!sections || index < 0 || index >= sections.length) {
            return null;
        }

        const createDocument = sections[index]?.createDocument;
        if (!createDocument) {
            return null;
        }

        const doc = await createDocument();
        return doc ?? null;
    })();

    if (cache) {
        cache.set(index, load);
    }

    return load;
}

export function resolveCfiFromParsedPositionsInDocument(
    view: ReaderTargetingView,
    start: KoReaderPosition,
    end: KoReaderPosition | null,
    doc: Document,
): string | null {
    const startNode = resolveTextNodeForPath(doc, start.nodePath);
    if (!startNode) {
        return null;
    }

    const endNode = end ? resolveTextNodeForPath(doc, end.nodePath) : startNode;
    if (!endNode) {
        return null;
    }

    const startOffset = clampTextOffset(startNode, start.offset);
    const endOffset = clampTextOffset(endNode, end?.offset ?? start.offset);

    const range = doc.createRange();
    try {
        range.setStart(startNode, startOffset);
        range.setEnd(endNode, endOffset);
    } catch {
        return null;
    }

    if (range.collapsed) {
        const startLength = startNode.nodeValue?.length ?? 0;
        if (startOffset < startLength) {
            range.setEnd(startNode, startOffset + 1);
        } else if (startOffset > 0) {
            range.setStart(startNode, startOffset - 1);
        }
    }

    if (range.collapsed) {
        return null;
    }

    try {
        return view.getCFI(start.spineIndex, range);
    } catch {
        return null;
    }
}

function parsePositionPair(
    pos0: string,
    pos1?: string | null,
): { start: KoReaderPosition; end: KoReaderPosition | null } | null {
    const start = parseKoReaderPosition(pos0);
    if (!start) {
        return null;
    }

    const end = pos1 ? parseKoReaderPosition(pos1) : null;
    return { start, end };
}

export function resolveCfiFromKoReaderPositionsInDocument(
    view: ReaderTargetingView,
    pos0: string,
    pos1: string | null | undefined,
    sectionIndex: number,
    doc: Document,
): string | null {
    const parsed = parsePositionPair(pos0, pos1);
    if (!parsed || parsed.start.spineIndex !== sectionIndex) {
        return null;
    }

    if (parsed.end && parsed.end.spineIndex !== sectionIndex) {
        return null;
    }

    return resolveCfiFromParsedPositionsInDocument(
        view,
        parsed.start,
        parsed.end,
        doc,
    );
}

export async function resolveCfiFromKoReaderPositions(
    view: ReaderTargetingView,
    pos0: string,
    pos1?: string | null,
    cache?: SectionDocumentCache,
): Promise<string | null> {
    const parsed = parsePositionPair(pos0, pos1);
    if (!parsed) {
        return null;
    }

    if (parsed.end && parsed.end.spineIndex !== parsed.start.spineIndex) {
        return null;
    }

    const doc = await sectionDocumentAt(view, parsed.start.spineIndex, cache);
    if (!doc) {
        return null;
    }

    return resolveCfiFromParsedPositionsInDocument(
        view,
        parsed.start,
        parsed.end,
        doc,
    );
}
