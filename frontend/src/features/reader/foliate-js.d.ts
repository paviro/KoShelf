declare module '@xincmm/foliate-js' {
    type OverlayerDrawFunction = (
        rects: DOMRectList | ArrayLike<DOMRect>,
        options?: Record<string, unknown>,
    ) => SVGElement;

    export class View extends HTMLElement {
        open(book: File | Blob | string): Promise<void>;
        init(opts: {
            lastLocation?: string;
            showTextStart?: boolean;
        }): Promise<void>;
        close(): void;
        goTo(target: string | number): Promise<void>;
        next(): Promise<void>;
        prev(): Promise<void>;
        addAnnotation(annotation: {
            value: string;
            color?: string;
            drawer?: string;
        }): Promise<{ index: number; label: string } | undefined>;
        deleteAnnotation(annotation: {
            value: string;
            color?: string;
            drawer?: string;
        }): Promise<void>;
        getCFI(index: number, range: Range): string;
        resolveNavigation(
            target: string | number,
        ):
            | { index: number; anchor?: (doc: Document) => Range | Element }
            | undefined;
        search(opts: {
            query: string;
            index?: number;
            matchCase?: boolean;
            matchDiacritics?: boolean;
            matchWholeWords?: boolean;
            defaultLocale?: string;
        }): AsyncGenerator<Record<string, unknown> | 'done'>;
        book?: {
            metadata?: { title?: string; language?: string };
            sections?: Array<{ id: string; linear?: string }>;
            toc?: Array<{ href: string; label: string }>;
            pageList?: Array<{ href: string; label: string }>;
        };
        renderer?: {
            setStyles?(css: string): void;
            setAttribute?(name: string, value: string): void;
            next(): Promise<void>;
            prev(): Promise<void>;
        };
        lastLocation?: {
            fraction?: number;
            tocItem?: { label?: string } | null;
            section?: { current: number; total: number } | null;
            cfi?: string;
        };
    }

    export class Overlayer {
        add(
            key: string,
            range: Range,
            draw: OverlayerDrawFunction,
            opts?: Record<string, unknown>,
        ): void;
        remove(key: string): void;
        hitTest(event: Event): [string | null, Range | null];
        static highlight: OverlayerDrawFunction;
        static underline: OverlayerDrawFunction;
        static outline: OverlayerDrawFunction;
    }
}

declare module '@xincmm/foliate-js/view.js' {
    export { View } from '@xincmm/foliate-js';
}

declare module '@xincmm/foliate-js/overlayer.js' {
    export { Overlayer } from '@xincmm/foliate-js';
}
