import type {
    ReaderTargetingView,
    SectionDocumentCache,
} from '../model/reader-model';
import { sectionDocumentAt } from './reader-cfi-resolution';

type TextSegment = {
    node: Text;
    start: number;
    end: number;
};

function escapeRegex(value: string): string {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function collectTextSegments(root: Node): {
    combined: string;
    segments: TextSegment[];
} {
    const ownerDocument = root.ownerDocument;
    if (!ownerDocument) {
        return { combined: '', segments: [] };
    }

    const walker = ownerDocument.createTreeWalker(root, NodeFilter.SHOW_TEXT);

    const segments: TextSegment[] = [];
    let combined = '';
    while (walker.nextNode()) {
        const node = walker.currentNode as Text;
        const text = node.nodeValue ?? '';
        if (!text) {
            continue;
        }

        const start = combined.length;
        combined += text;
        segments.push({
            node,
            start,
            end: combined.length,
        });
    }

    return { combined, segments };
}

function locateTextPosition(
    segments: TextSegment[],
    absoluteOffset: number,
): { node: Text; offset: number } | null {
    for (let i = 0; i < segments.length; i += 1) {
        const segment = segments[i];
        if (absoluteOffset < segment.end) {
            return {
                node: segment.node,
                offset: absoluteOffset - segment.start,
            };
        }
    }

    const last = segments[segments.length - 1];
    if (!last) {
        return null;
    }

    return {
        node: last.node,
        offset: last.node.nodeValue?.length ?? 0,
    };
}

function findRangeByText(doc: Document, query: string): Range | null {
    const normalizedQuery = query.trim();
    if (!normalizedQuery) {
        return null;
    }

    const root = doc.body ?? doc.documentElement;
    if (!root) {
        return null;
    }

    const { combined, segments } = collectTextSegments(root);
    if (!combined || segments.length === 0) {
        return null;
    }

    const lowerCombined = combined.toLowerCase();
    const lowerQuery = normalizedQuery.toLowerCase();

    let start = lowerCombined.indexOf(lowerQuery);
    let end = start >= 0 ? start + lowerQuery.length : -1;

    if (start < 0) {
        const whitespacePattern = escapeRegex(normalizedQuery).replace(
            /\s+/g,
            '\\s+',
        );
        const regex = new RegExp(whitespacePattern, 'i');
        const match = regex.exec(combined);
        if (!match || typeof match.index !== 'number') {
            return null;
        }

        start = match.index;
        end = start + match[0].length;
    }

    const startPos = locateTextPosition(segments, start);
    const endPos = locateTextPosition(segments, end);
    if (!startPos || !endPos) {
        return null;
    }

    const range = doc.createRange();
    range.setStart(startPos.node, startPos.offset);
    range.setEnd(endPos.node, endPos.offset);

    if (range.collapsed) {
        return null;
    }

    return range;
}

export function resolveCfiByTextInDocument(
    view: ReaderTargetingView,
    text: string,
    index: number,
    doc: Document,
): string | null {
    const range = findRangeByText(doc, text);
    if (!range) {
        return null;
    }

    try {
        return view.getCFI(index, range);
    } catch {
        return null;
    }
}

export async function resolveCfiByTextInSection(
    view: ReaderTargetingView,
    text: string,
    index: number,
    cache?: SectionDocumentCache,
): Promise<string | null> {
    const doc = await sectionDocumentAt(view, index, cache);
    if (!doc) {
        return null;
    }

    return resolveCfiByTextInDocument(view, text, index, doc);
}

export async function resolveCfiByTextAcrossSections(
    view: ReaderTargetingView,
    text: string,
    cache?: SectionDocumentCache,
): Promise<string | null> {
    const sectionCount = view.book?.sections?.length ?? 0;
    for (let index = 0; index < sectionCount; index += 1) {
        const cfi = await resolveCfiByTextInSection(view, text, index, cache);
        if (cfi) {
            return cfi;
        }
    }

    return null;
}
