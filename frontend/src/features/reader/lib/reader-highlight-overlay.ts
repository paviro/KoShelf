type AnnotationRenderer = (
    rects: DOMRectList | ArrayLike<DOMRect>,
    opts?: Record<string, unknown>,
) => SVGElement;

export type HighlightRenderers = {
    highlight: AnnotationRenderer;
    underline: AnnotationRenderer;
};

type DrawAnnotationDetail = {
    draw?: (
        renderer: AnnotationRenderer,
        opts?: Record<string, unknown>,
    ) => void;
    annotation?: { color?: string; drawer?: string };
};

const DRAWER_TO_RENDERER: Record<string, keyof HighlightRenderers> = {
    lighten: 'highlight',
    invert: 'highlight',
    underscore: 'underline',
    strikeout: 'underline',
};

export function attachHighlightDrawListener(
    view: EventTarget,
    renderers: HighlightRenderers,
    defaultColor: string = '#eab308',
): () => void {
    const listener = (event: Event) => {
        const detail = (event as CustomEvent).detail as
            | DrawAnnotationDetail
            | undefined;
        if (typeof detail?.draw !== 'function') {
            return;
        }

        const color = detail.annotation?.color ?? defaultColor;
        const rendererKey =
            DRAWER_TO_RENDERER[detail.annotation?.drawer ?? ''] ?? 'highlight';
        detail.draw(renderers[rendererKey], { color });
    };

    view.addEventListener('draw-annotation', listener as EventListener);

    return () => {
        view.removeEventListener('draw-annotation', listener as EventListener);
    };
}
