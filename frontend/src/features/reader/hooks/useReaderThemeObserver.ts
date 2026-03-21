import { useEffect, type RefObject } from 'react';

import type { LibraryReaderPresentation } from '../../library/api/library-data';
import type { FoliateView } from '../model/reader-model';
import { applyReaderPresentation } from '../lib/reader-theme';

export function useReaderThemeObserver(
    viewRef: RefObject<FoliateView | null>,
    fontSizePt: number,
    lineSpacing: number,
    presentation: LibraryReaderPresentation | null | undefined,
): void {
    useEffect(() => {
        const rootElement = document.documentElement;
        const updateReaderStyles = () => {
            const currentView = viewRef.current;
            if (!currentView) {
                return;
            }
            applyReaderPresentation(
                currentView,
                fontSizePt,
                lineSpacing,
                presentation,
            );
        };

        const observer = new MutationObserver(updateReaderStyles);
        const currentRenderer = viewRef.current?.renderer;
        const resizeObserver =
            currentRenderer && typeof ResizeObserver !== 'undefined'
                ? new ResizeObserver(updateReaderStyles)
                : null;

        updateReaderStyles();

        if (currentRenderer && resizeObserver) {
            resizeObserver.observe(currentRenderer);
        }

        observer.observe(rootElement, {
            attributes: true,
            attributeFilter: ['class', 'style'],
        });
        window.addEventListener('resize', updateReaderStyles);

        return () => {
            observer.disconnect();
            resizeObserver?.disconnect();
            window.removeEventListener('resize', updateReaderStyles);
        };
    }, [viewRef, fontSizePt, lineSpacing, presentation]);
}
