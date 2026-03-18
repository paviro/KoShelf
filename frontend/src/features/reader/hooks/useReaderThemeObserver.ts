import { useEffect, type RefObject } from 'react';

import type { FoliateView } from '../model/reader-model';
import { applyReaderPresentation } from '../lib/reader-theme';

export function useReaderThemeObserver(
    viewRef: RefObject<FoliateView | null>,
): void {
    useEffect(() => {
        const rootElement = document.documentElement;
        const updateReaderStyles = () => {
            const currentView = viewRef.current;
            if (!currentView) {
                return;
            }
            applyReaderPresentation(currentView);
        };

        const observer = new MutationObserver((entries) => {
            if (entries.length > 0) {
                updateReaderStyles();
            }
        });

        observer.observe(rootElement, {
            attributes: true,
            attributeFilter: ['class', 'style'],
        });

        return () => {
            observer.disconnect();
        };
    }, [viewRef]);
}
