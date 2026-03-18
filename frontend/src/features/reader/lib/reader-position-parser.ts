import type { KoReaderPosition } from '../model/reader-model';

const KOREADER_POSITION_RE =
    /^\/body\/(?:DocFragment|section)\[(\d+)\](\/.*)\.(\d+)$/;

export function parseKoReaderPosition(pos: string): KoReaderPosition | null {
    const match = pos.match(KOREADER_POSITION_RE);
    if (!match) {
        return null;
    }

    const spineIndex = Number.parseInt(match[1], 10) - 1;
    const offset = Number.parseInt(match[3], 10);
    if (Number.isNaN(spineIndex) || spineIndex < 0 || Number.isNaN(offset)) {
        return null;
    }

    return {
        spineIndex,
        nodePath: match[2],
        offset,
    };
}

export function clampTextOffset(node: Text, offset: number): number {
    if (offset <= 0) {
        return 0;
    }

    const maxOffset = node.nodeValue?.length ?? 0;
    return offset > maxOffset ? maxOffset : offset;
}

function parseElementStep(step: string): { name: string; nth: number } | null {
    const match = step.match(/^([A-Za-z0-9_-]+)(?:\[(\d+)\])?$/);
    if (!match) {
        return null;
    }

    const nth = match[2] ? Number.parseInt(match[2], 10) : 1;
    if (Number.isNaN(nth) || nth <= 0) {
        return null;
    }

    return {
        name: match[1].toLowerCase(),
        nth,
    };
}

function parseTextStep(step: string): number | null {
    const match = step.match(/^text\(\)(?:\[(\d+)\])?$/i);
    if (!match) {
        return null;
    }

    const nth = match[1] ? Number.parseInt(match[1], 10) : 1;
    if (Number.isNaN(nth) || nth <= 0) {
        return null;
    }

    return nth;
}

function childElementAt(
    parent: Element,
    name: string,
    nth: number,
): Element | null {
    let count = 0;
    for (const childNode of parent.childNodes) {
        if (
            childNode.nodeType === Node.ELEMENT_NODE &&
            childNode.nodeName.toLowerCase() === name
        ) {
            count += 1;
            if (count === nth) {
                return childNode as Element;
            }
        }
    }

    return null;
}

function childTextAt(parent: Node, nth: number): Text | null {
    let count = 0;
    for (const childNode of parent.childNodes) {
        if (childNode.nodeType === Node.TEXT_NODE) {
            count += 1;
            if (count === nth) {
                return childNode as Text;
            }
        }
    }

    return null;
}

export function resolveTextNodeForPath(
    doc: Document,
    nodePath: string,
): Text | null {
    const steps = nodePath.split('/').filter(Boolean);
    if (steps.length === 0) {
        return null;
    }

    let current: Node | null = doc.body ?? doc.documentElement;
    if (!current) {
        return null;
    }

    if (steps[0].toLowerCase() === 'body') {
        steps.shift();
    }

    if (steps.length === 0) {
        return null;
    }

    for (let i = 0; i < steps.length; i += 1) {
        const step = steps[i];
        const textNth = parseTextStep(step);
        if (textNth !== null) {
            return childTextAt(current, textNth);
        }

        if (current.nodeType !== Node.ELEMENT_NODE) {
            return null;
        }

        const parsedElementStep = parseElementStep(step);
        if (!parsedElementStep) {
            return null;
        }

        current = childElementAt(
            current as Element,
            parsedElementStep.name,
            parsedElementStep.nth,
        );

        if (!current) {
            return null;
        }
    }

    return null;
}
