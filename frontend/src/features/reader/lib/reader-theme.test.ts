import { describe, expect, it, vi } from 'vitest';

import type { FoliateRenderer, FoliateView } from '../model/reader-model';
import {
    applyReaderPresentation,
    mapKoReaderFontSizePtToCssPercent,
} from './reader-theme';

function extractAppliedFontSizePercent(
    setStyles: ReturnType<typeof vi.fn>,
): number {
    const latestCall = setStyles.mock.calls.at(-1);
    const appliedStyles = latestCall?.[0];

    if (!Array.isArray(appliedStyles)) {
        throw new Error('Expected reader style tuple.');
    }

    const [baseStyles] = appliedStyles;
    const fontSizeMatch = baseStyles.match(
        /font-size:\s*([0-9.]+)%\s*!important;/,
    );
    if (!fontSizeMatch) {
        throw new Error('Reader font-size declaration was not applied.');
    }

    return Number(fontSizeMatch[1]);
}

describe('applyReaderPresentation', () => {
    it('keeps the applied font size stable when renderer dimensions change', () => {
        const setStyles = vi.fn();
        const renderer = document.createElement('div') as FoliateRenderer;
        renderer.setStyles = setStyles;

        let width = 1200;
        let height = 900;
        Object.defineProperty(renderer, 'getBoundingClientRect', {
            value: () => ({
                x: 0,
                y: 0,
                width,
                height,
                top: 0,
                left: 0,
                right: width,
                bottom: height,
                toJSON: () => ({}),
            }),
        });

        const view = document.createElement('div') as unknown as FoliateView;
        view.renderer = renderer;

        applyReaderPresentation(view, 22, 1.5, null);
        const initialFontSize = extractAppliedFontSizePercent(setStyles);

        width = 640;
        height = 420;
        applyReaderPresentation(view, 22, 1.5, null);
        const resizedFontSize = extractAppliedFontSizePercent(setStyles);

        expect(initialFontSize).toBe(mapKoReaderFontSizePtToCssPercent(22));
        expect(resizedFontSize).toBe(initialFontSize);
    });
});
