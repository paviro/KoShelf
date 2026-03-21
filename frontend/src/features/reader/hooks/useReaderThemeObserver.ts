import { useEffect, type RefObject } from 'react';

import type { FoliateView } from '../model/reader-model';
import { applyReaderPresentation } from '../lib/reader-theme';

export function useReaderThemeObserver(
    viewRef: RefObject<FoliateView | null>,
    fontSize: number,
    lineSpacing: number,
): void {
    useEffect(() => {
        const rootElement = document.documentElement;
        const updateReaderStyles = () => {
            const currentView = viewRef.current;
            if (!currentView) {
                return;
            }
            applyReaderPresentation(currentView, fontSize, lineSpacing);
        };

        const observer = new MutationObserver(updateReaderStyles);

        updateReaderStyles();

        observer.observe(rootElement, {
            attributes: true,
            attributeFilter: ['class', 'style'],
        });

        return () => {
            observer.disconnect();
        };
    }, [viewRef, fontSize, lineSpacing]);
}
