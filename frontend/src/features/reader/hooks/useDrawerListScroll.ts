import { useEffect, type RefObject } from 'react';

export function useDrawerListScroll(
    listRef: RefObject<HTMLElement | null>,
    currentIndex: number,
    dataAttribute: string,
): void {
    useEffect(() => {
        if (currentIndex < 0 || !listRef.current) {
            return;
        }

        const scrollContainer = listRef.current.closest<HTMLElement>(
            '[data-tabbed-drawer-scroll-container]',
        );
        if (scrollContainer) {
            scrollContainer.style.overflowY = 'hidden';
        }

        const restoreOverflow = () => {
            if (scrollContainer) {
                scrollContainer.style.overflowY = '';
            }
        };

        const scroll = () => {
            const el = listRef.current?.querySelector<HTMLElement>(
                `[${dataAttribute}]`,
            );
            if (!el) {
                return;
            }

            el.scrollIntoView({
                block: 'center',
                inline: 'nearest',
            });
        };

        const frameId = requestAnimationFrame(scroll);
        const timeoutId = setTimeout(scroll, 350);
        const restoreOverflowId = window.setTimeout(restoreOverflow, 425);
        return () => {
            cancelAnimationFrame(frameId);
            clearTimeout(timeoutId);
            window.clearTimeout(restoreOverflowId);
            restoreOverflow();
        };
    }, [currentIndex, dataAttribute, listRef]);
}
