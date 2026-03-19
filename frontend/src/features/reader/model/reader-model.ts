import type { LibraryCollection } from '../../library/model/library-model';

export type ReaderRouteProps = {
    collection: LibraryCollection;
};

export type ReaderLocation = {
    fraction: number;
    tocItem?: { label?: string; href?: string } | null;
    section?: { current: number; total: number } | null;
};

export type FoliateRenderer = HTMLElement & {
    setStyles?: (styles: string | [string, string]) => void;
};

export type FoliateAnnotation = {
    value: string;
    color?: string;
    drawer?: string;
};

export type FoliateAnnotationResult =
    | {
          index: number;
          label: string;
      }
    | undefined;

export type FoliateView = HTMLElement &
    ReaderTargetingView & {
        isFixedLayout?: boolean;
        renderer?: FoliateRenderer;
        lastLocation?: { cfi?: string };
        open: (book: File | Blob | string) => Promise<void>;
        init: (opts: {
            lastLocation?: string;
            showTextStart?: boolean;
        }) => Promise<void>;
        close: () => void;
        goTo: (target: ReaderNavigationTarget) => Promise<void>;
        goToFraction: (frac: number) => Promise<void>;
        next: () => Promise<void>;
        prev: () => Promise<void>;
        addAnnotation: (
            annotation: FoliateAnnotation,
        ) => Promise<FoliateAnnotationResult>;
    };

type ReaderSection = {
    createDocument?: (() => Promise<Document> | Document) | null;
};

export type TocEntry = {
    href: string;
    label: string;
    sectionIndex?: number;
    depth: number;
};

type ReaderBookTocEntry = {
    href?: string | null;
    label?: string | null;
    subitems?: ReaderBookTocEntry[] | null;
};

type ReaderBook = {
    sections?: ReaderSection[];
    toc?: ReaderBookTocEntry[];
    pageList?: ReaderBookTocEntry[];
};

type ResolvedNavigation = {
    index?: number;
};

export type ReaderTargetingView = {
    book?: ReaderBook;
    getCFI: (index: number, range: Range) => string;
    resolveNavigation: (target: string) => ResolvedNavigation | undefined;
};

export type KoReaderPosition = {
    spineIndex: number;
    nodePath: string;
    offset: number;
};

export type ReaderNavigationTarget = string | number;

export type ReaderHighlightValue = {
    value: string;
    color?: string;
    drawer?: string;
    note?: string;
};

export type ResolveHighlightsBySectionOptions = {
    prioritizeSectionIndexes?: number[];
    maxConcurrentSections?: number;
    onSectionResolved?: (
        sectionIndex: number,
        sectionHighlights: ReaderHighlightValue[],
    ) => void | Promise<void>;
};

export type SectionDocumentCache = Map<number, Promise<Document | null>>;
