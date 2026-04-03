import { describe, expect, it, vi } from 'vitest';

import { attachHighlightDrawListener } from './reader-highlight-overlay';
import type { HighlightRenderers } from './reader-highlight-overlay';

function makeRenderers(): HighlightRenderers {
    return {
        highlight: vi.fn(),
        underline: vi.fn(),
        strikethrough: vi.fn(),
        invert: vi.fn(),
    };
}

describe('attachHighlightDrawListener', () => {
    it('registers draw callback with highlight renderer and color', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers, '#abc123');

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw },
            }),
        );

        expect(draw).toHaveBeenCalledTimes(1);
        expect(draw).toHaveBeenCalledWith(renderers.highlight, {
            color: '#abc123',
        });

        handle.detach();
    });

    it('uses per-annotation color when present', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { color: '#ff0000' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.highlight, {
            color: '#ff0000',
        });

        handle.detach();
    });

    it('selects underline renderer for underscore drawer', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { drawer: 'underscore' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.underline, {
            color: '#eab308',
        });

        handle.detach();
    });

    it('selects strikethrough renderer for strikeout drawer', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { drawer: 'strikeout' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.strikethrough, {
            color: '#eab308',
        });

        handle.detach();
    });

    it('selects invert renderer for invert drawer', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { drawer: 'invert' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.invert, {
            color: '#eab308',
        });

        handle.detach();
    });

    it('detaches listener cleanly', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);
        handle.detach();

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw },
            }),
        );

        expect(draw).not.toHaveBeenCalled();
    });

    it('ignores events without draw function', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const handle = attachHighlightDrawListener(view, renderers);

        expect(() => {
            view.dispatchEvent(new CustomEvent('draw-annotation'));
            view.dispatchEvent(
                new CustomEvent('draw-annotation', {
                    detail: { draw: null },
                }),
            );
        }).not.toThrow();

        handle.detach();
    });

    it('uses pulse renderer for targeted annotation', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: {
                    draw,
                    annotation: {
                        target: true,
                        value: 'cfi-1',
                        color: '#ff0000',
                    },
                },
            }),
        );

        expect(draw).toHaveBeenCalledTimes(1);
        const wrappedRenderer = draw.mock.calls[0][0];
        expect(wrappedRenderer).not.toBe(renderers.highlight);
        expect(draw.mock.calls[0][1]).toEqual({ color: '#ff0000' });

        handle.detach();
    });

    it('does not re-pulse targeted annotation on second draw', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        const annotation = {
            target: true,
            value: 'cfi-1',
        };

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation },
            }),
        );
        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation },
            }),
        );

        expect(draw).toHaveBeenCalledTimes(2);
        const firstRenderer = draw.mock.calls[0][0];
        const secondRenderer = draw.mock.calls[1][0];
        expect(firstRenderer).not.toBe(renderers.highlight);
        expect(secondRenderer).toBe(renderers.highlight);

        handle.detach();
    });

    it('uses plain renderer for non-targeted annotations', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const handle = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: {
                    draw,
                    annotation: { value: 'cfi-2', color: '#00ff00' },
                },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.highlight, {
            color: '#00ff00',
        });

        handle.detach();
    });
});
