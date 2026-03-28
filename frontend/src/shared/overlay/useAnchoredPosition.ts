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

        document.addEventListener('scroll', reposition, {
            passive: true,
            capture: true,
        });
        window.addEventListener('resize', reposition, { passive: true });
        return () => {
            document.removeEventListener('scroll', reposition, true);
            window.removeEventListener('resize', reposition);
        };
    }, [active, reposition]);
}
