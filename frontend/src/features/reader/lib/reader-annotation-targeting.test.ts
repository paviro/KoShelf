import { describe, expect, it, vi } from 'vitest';

import type { LibraryAnnotation } from '../../library/api/library-data';
import {
    resolveAnnotationCfi,
    resolveAnnotationTarget,
} from './reader-navigation-resolution';
import { resolveHighlightsBySection } from './reader-highlight-resolution';
import { parseKoReaderPosition } from './reader-position-parser';

function makeDoc(bodyHtml: string): Document {
    const parser = new DOMParser();
    return parser.parseFromString(
        `<html><body>${bodyHtml}</body></html>`,
        'text/html',
    );
}

function makeView(docs: Document[]) {
    return {
        book: {
            sections: docs.map((doc) => ({
                createDocument: () => doc,
            })),
        },
        getCFI: (index: number, range: Range) =>
            `cfi-${index}:${range.toString()}`,
        resolveNavigation: (target: string) => {
            const match = target.match(/^cfi-(\d+):/);
            if (!match) {
                return undefined;
            }

            return {
                index: Number.parseInt(match[1], 10),
            };
        },
    };
}

describe('reader annotation targeting', () => {
    it('parses KoReader position with node path and offset', () => {
        const parsed = parseKoReaderPosition(
            '/body/DocFragment[14]/body/section/p[3]/text()[2].106',
        );

        expect(parsed).toEqual({
            spineIndex: 13,
            nodePath: '/body/section/p[3]/text()[2]',
            offset: 106,
        });
    });

    it('resolves CFI directly from pos0/pos1', async () => {
        const view = makeView([
            makeDoc('<section><p>abcdefghij</p></section>'),
        ]);

        const annotation: LibraryAnnotation = {
            pos0: '/body/DocFragment[1]/body/section/p[1]/text().2',
            pos1: '/body/DocFragment[1]/body/section/p[1]/text().6',
        };

        const cfi = await resolveAnnotationCfi(view, annotation);

        expect(cfi).toBe('cfi-0:cdef');
    });

    it('falls back to section text search when position fails', async () => {
        const view = makeView([
            makeDoc(
                '<section><p>Hello\n   world from reader tests.</p></section>',
            ),
        ]);

        const annotation: LibraryAnnotation = {
            pos0: '/body/DocFragment[999]/body/section/p[1]/text().0',
            text: 'hello world',
        };

        const cfi = await resolveAnnotationCfi(view, annotation);

        expect(cfi).toBe('cfi-0:Hello\n   world');
    });

    it('resolves chapter href when no CFI exists', async () => {
        const view = {
            ...makeView([]),
            book: {
                sections: [],
                toc: [{ label: 'My Topic', href: '#topic' }],
            },
        };

        const annotation: LibraryAnnotation = {
            chapter: 'Chapter 12: My Topic',
        };

        const target = await resolveAnnotationTarget(view, annotation);

        expect(target).toBe('#topic');
    });

    it('resolves page href when chapter is unavailable', async () => {
        const view = {
            ...makeView([]),
            book: {
                sections: [],
                pageList: [{ label: '42', href: '#p42' }],
            },
        };

        const annotation: LibraryAnnotation = {
            pageno: 42,
        };

        const target = await resolveAnnotationTarget(view, annotation);

        expect(target).toBe('#p42');
    });

    it('falls back to section index navigation target', async () => {
        const view = {
            ...makeView([makeDoc('<section><p>x</p></section>')]),
            getCFI: () => {
                throw new Error('Cannot compute CFI');
            },
        };

        const annotation: LibraryAnnotation = {
            pos0: '/body/DocFragment[1]/body/section/p[1]/text().0',
        };

        const target = await resolveAnnotationTarget(view, annotation);

        expect(target).toBe(0);
    });

    it('groups resolved highlights by section for overlay rendering', async () => {
        const view = makeView([
            makeDoc('<section><p>first section text</p></section>'),
            makeDoc('<section><p>second section text</p></section>'),
        ]);

        const highlights: LibraryAnnotation[] = [
            {
                pos0: '/body/DocFragment[1]/body/section/p[1]/text().0',
                pos1: '/body/DocFragment[1]/body/section/p[1]/text().5',
            },
            {
                pos0: '/body/DocFragment[2]/body/section/p[1]/text().0',
                pos1: '/body/DocFragment[2]/body/section/p[1]/text().6',
            },
            {
                text: 'not found',
            },
        ];

        const bySection = await resolveHighlightsBySection(
            view,
            highlights,
            false,
        );

        expect(bySection.get(0)).toEqual([
            { value: 'cfi-0:first', color: '#eab308' },
        ]);
        expect(bySection.get(1)).toEqual([
            { value: 'cfi-1:second', color: '#eab308' },
        ]);
        expect(bySection.size).toBe(2);
    });

    it('reuses section documents for multiple highlights in same section', async () => {
        const doc = makeDoc('<section><p>abcdefghij</p></section>');
        const createDocument = vi.fn(() => doc);
        const view = {
            book: {
                sections: [{ createDocument }],
            },
            getCFI: (index: number, range: Range) =>
                `cfi-${index}:${range.toString()}`,
            resolveNavigation: (target: string) => {
                const match = target.match(/^cfi-(\d+):/);
                if (!match) {
                    return undefined;
                }

                return {
                    index: Number.parseInt(match[1], 10),
                };
            },
        };

        const highlights: LibraryAnnotation[] = [
            {
                pos0: '/body/DocFragment[1]/body/section/p[1]/text().0',
                pos1: '/body/DocFragment[1]/body/section/p[1]/text().2',
            },
            {
                pos0: '/body/DocFragment[1]/body/section/p[1]/text().2',
                pos1: '/body/DocFragment[1]/body/section/p[1]/text().4',
            },
            {
                pos0: '/body/DocFragment[1]/body/section/p[1]/text().4',
                pos1: '/body/DocFragment[1]/body/section/p[1]/text().6',
            },
        ];

        const bySection = await resolveHighlightsBySection(
            view,
            highlights,
            false,
        );

        expect(createDocument).toHaveBeenCalledTimes(1);
        expect(bySection.get(0)).toEqual([
            { value: 'cfi-0:ab', color: '#eab308' },
            { value: 'cfi-0:cd', color: '#eab308' },
            { value: 'cfi-0:ef', color: '#eab308' },
        ]);
    });

    it('prioritizes selected sections during resolution', async () => {
        const view = makeView([
            makeDoc('<section><p>first section text</p></section>'),
            makeDoc('<section><p>second section text</p></section>'),
        ]);

        const highlights: LibraryAnnotation[] = [
            {
                pos0: '/body/DocFragment[1]/body/section/p[1]/text().0',
                pos1: '/body/DocFragment[1]/body/section/p[1]/text().5',
            },
            {
                pos0: '/body/DocFragment[2]/body/section/p[1]/text().0',
                pos1: '/body/DocFragment[2]/body/section/p[1]/text().6',
            },
        ];

        const resolvedSections: number[] = [];
        const bySection = await resolveHighlightsBySection(
            view,
            highlights,
            false,
            {
                prioritizeSectionIndexes: [1],
                maxConcurrentSections: 1,
                onSectionResolved: (sectionIndex) => {
                    resolvedSections.push(sectionIndex);
                },
            },
        );

        expect(resolvedSections).toEqual([1, 0]);
        expect(bySection.get(0)).toEqual([
            { value: 'cfi-0:first', color: '#eab308' },
        ]);
        expect(bySection.get(1)).toEqual([
            { value: 'cfi-1:second', color: '#eab308' },
        ]);
    });
});
