import type { LibraryCollection } from '../../library/model/library-model';

export type ReaderRouteProps = {
    collection: LibraryCollection;
};

export type ReaderLocation = {
    fraction: number;
    tocItem?: { label?: string } | null;
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

type TocEntry = {
    href: string;
    label: string;
};

type ReaderBook = {
    sections?: ReaderSection[];
    toc?: TocEntry[];
    pageList?: TocEntry[];
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
