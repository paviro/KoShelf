import { useLayoutEffect } from 'react';
import { useLocation } from 'react-router-dom';

const LIBRARY_LIST_PATHS = new Set(['/books', '/comics']);

/**
 * Scrolls the window to the top on every pathname change, except for
 * library list routes which manage their own scroll restoration.
 *
 * Temporarily hides overflow before resetting so the macOS overlay
 * scrollbar doesn't flash during the programmatic scroll.
 */
export function ScrollToTop(): null {
    const { pathname } = useLocation();

    useLayoutEffect(() => {
        if (LIBRARY_LIST_PATHS.has(pathname) || window.scrollY === 0) {
            return;
        }

        const html = document.documentElement;
        html.style.overflow = 'hidden';
        window.scrollTo(0, 0);

        const frameId = requestAnimationFrame(() => {
            html.style.overflow = '';
        });

        return () => {
            cancelAnimationFrame(frameId);
            html.style.overflow = '';
        };
    }, [pathname]);

    return null;
}
