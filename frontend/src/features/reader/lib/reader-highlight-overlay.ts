type AnnotationRenderer = (
    rects: DOMRectList | ArrayLike<DOMRect>,
    opts?: Record<string, unknown>,
) => SVGElement;

export type HighlightRenderers = {
    highlight: AnnotationRenderer;
    underline: AnnotationRenderer;
    strikethrough: AnnotationRenderer;
    invert: AnnotationRenderer;
};

export type HighlightDrawListenerHandle = {
    detach: () => void;
    clearAnimatedValue: (value: string) => void;
};

type DrawAnnotationDetail = {
    draw?: (
        renderer: AnnotationRenderer,
        opts?: Record<string, unknown>,
    ) => void;
    annotation?: {
        color?: string;
        drawer?: string;
        target?: boolean;
        value?: string;
    };
};

const DRAWER_TO_RENDERER: Record<string, keyof HighlightRenderers> = {
    lighten: 'highlight',
    invert: 'invert',
    underscore: 'underline',
    strikeout: 'strikethrough',
};

function withPulse(renderer: AnnotationRenderer): AnnotationRenderer {
    const createdAt = Date.now();
    return (rects, opts) => {
        const element = renderer(rects, opts);
        if (Date.now() - createdAt > 3000) {
            return element;
        }
        const restingOpacity = element.style.opacity;
        element.style.transition = 'opacity 0.4s ease-in-out';
        setTimeout(() => (element.style.opacity = '0.75'), 50);
        setTimeout(() => {
            element.style.transition = 'opacity 0.7s ease-out';
            element.style.opacity = restingOpacity;
        }, 450);
        return element;
    };
}

export function attachHighlightDrawListener(
    view: EventTarget,
    renderers: HighlightRenderers,
    defaultColor: string = '#eab308',
): HighlightDrawListenerHandle {
    const animatedValues = new Set<string>();

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
        const baseRenderer = renderers[rendererKey];

        const annotationValue = detail.annotation?.value;
        const shouldAnimate =
            detail.annotation?.target === true &&
            typeof annotationValue === 'string' &&
            !animatedValues.has(annotationValue);

        if (shouldAnimate) {
            animatedValues.add(annotationValue);
            detail.draw(withPulse(baseRenderer), { color });
            if (detail.annotation) {
                detail.annotation.target = false;
            }
        } else {
            detail.draw(baseRenderer, { color });
        }
    };

    view.addEventListener('draw-annotation', listener as EventListener);

    return {
        detach: () => {
            view.removeEventListener(
                'draw-annotation',
                listener as EventListener,
            );
        },
        clearAnimatedValue: (value: string) => {
            animatedValues.delete(value);
        },
    };
}
