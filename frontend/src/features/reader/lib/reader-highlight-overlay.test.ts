import { describe, expect, it, vi } from 'vitest';

import { attachHighlightDrawListener } from './reader-highlight-overlay';
import type { HighlightRenderers } from './reader-highlight-overlay';

function makeRenderers(): HighlightRenderers {
    return {
        highlight: vi.fn(),
        underline: vi.fn(),
    };
}

describe('attachHighlightDrawListener', () => {
    it('registers draw callback with highlight renderer and color', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const detach = attachHighlightDrawListener(view, renderers, '#abc123');

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw },
            }),
        );

        expect(draw).toHaveBeenCalledTimes(1);
        expect(draw).toHaveBeenCalledWith(renderers.highlight, {
            color: '#abc123',
        });

        detach();
    });

    it('uses per-annotation color when present', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const detach = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { color: '#ff0000' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.highlight, {
            color: '#ff0000',
        });

        detach();
    });

    it('selects underline renderer for underscore drawer', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const detach = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { drawer: 'underscore' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.underline, {
            color: '#eab308',
        });

        detach();
    });

    it('selects underline renderer for strikeout drawer', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const detach = attachHighlightDrawListener(view, renderers);

        view.dispatchEvent(
            new CustomEvent('draw-annotation', {
                detail: { draw, annotation: { drawer: 'strikeout' } },
            }),
        );

        expect(draw).toHaveBeenCalledWith(renderers.underline, {
            color: '#eab308',
        });

        detach();
    });

    it('detaches listener cleanly', () => {
        const view = new EventTarget();
        const renderers = makeRenderers();
        const draw = vi.fn();

        const detach = attachHighlightDrawListener(view, renderers);
        detach();

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
        const detach = attachHighlightDrawListener(view, renderers);

        expect(() => {
            view.dispatchEvent(new CustomEvent('draw-annotation'));
            view.dispatchEvent(
                new CustomEvent('draw-annotation', {
                    detail: { draw: null },
                }),
            );
        }).not.toThrow();

        detach();
    });
});
