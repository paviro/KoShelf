import {
    useCallback,
    useEffect,
    useLayoutEffect,
    useRef,
    type RefObject,
} from 'react';

import {
    computeOverlayPosition,
    getShellInset,
    type OverlayPositioningOptions,
} from './anchored-overlay';

export function useAnchoredPosition(
    anchorRef: RefObject<HTMLElement | null>,
    overlayRef: RefObject<HTMLElement | null>,
    active: boolean,
    onClose: () => void,
    options?: OverlayPositioningOptions,
): void {
    const onCloseRef = useRef(onClose);
    const optionsRef = useRef(options);

    useEffect(() => {
        onCloseRef.current = onClose;
        optionsRef.current = options;
    });

    const reposition = useCallback(() => {
        const anchor = anchorRef.current;
        const overlay = overlayRef.current;
        if (!anchor || !overlay) return;

        const opts = optionsRef.current;
        const result = computeOverlayPosition(
            anchor.getBoundingClientRect(),
            overlay.getBoundingClientRect(),
            window.innerWidth,
            window.innerHeight,
            { ...opts, viewportInset: opts?.viewportInset ?? getShellInset() },
        );

        if (result.anchorOffScreen) {
            onCloseRef.current();
            return;
        }

        overlay.style.top = `${result.top}px`;
        overlay.style.left = `${result.left}px`;
        overlay.style.visibility = 'visible';
    }, [anchorRef, overlayRef]);

    useLayoutEffect(() => {
        if (active) {
            reposition();
        }
    }, [active, reposition]);

    useEffect(() => {
        if (!active) return;

        let rafId = 0;
        const scheduleReposition = () => {
            if (!rafId) {
                rafId = requestAnimationFrame(() => {
                    rafId = 0;
                    reposition();
                });
            }
        };

        document.addEventListener('scroll', scheduleReposition, {
            passive: true,
            capture: true,
        });
        window.addEventListener('resize', scheduleReposition, {
            passive: true,
        });
        return () => {
            cancelAnimationFrame(rafId);
            document.removeEventListener('scroll', scheduleReposition, true);
            window.removeEventListener('resize', scheduleReposition);
        };
    }, [active, reposition]);
}
