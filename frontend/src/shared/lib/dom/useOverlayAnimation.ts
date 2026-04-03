import { useEffect, useRef, useState } from 'react';

import { useEscapeKey } from './useEscapeKey';

export const OVERLAY_TRANSITION_DURATION_MS = 300;

type UseOverlayAnimationReturn = {
    isMounted: boolean;
    isVisible: boolean;
    backdropRef: React.RefObject<HTMLDivElement | null>;
};

export function useOverlayAnimation(
    open: boolean,
    onClose: () => void,
): UseOverlayAnimationReturn {
    const [isMounted, setIsMounted] = useState(false);
    const [isVisible, setIsVisible] = useState(false);
    const backdropRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (open) {
            const frameId = window.requestAnimationFrame(() => {
                setIsMounted(true);
            });

            return () => {
                window.cancelAnimationFrame(frameId);
            };
        }

        if (!isMounted) {
            return;
        }

        const frameId = window.requestAnimationFrame(() => {
            setIsVisible(false);
        });

        return () => {
            window.cancelAnimationFrame(frameId);
        };
    }, [isMounted, open]);

    useEffect(() => {
        if (!open && isMounted) {
            const timeoutId = window.setTimeout(() => {
                setIsMounted(false);
            }, OVERLAY_TRANSITION_DURATION_MS);
            return () => window.clearTimeout(timeoutId);
        }
    }, [open, isMounted]);

    useEffect(() => {
        if (!open || !isMounted) {
            return;
        }

        if (backdropRef.current) {
            void backdropRef.current.offsetHeight;
        }

        const frameId = window.requestAnimationFrame(() => {
            setIsVisible(true);
        });

        return () => {
            window.cancelAnimationFrame(frameId);
        };
    }, [isMounted, open]);

    useEscapeKey(onClose, isMounted);

    return { isMounted, isVisible, backdropRef };
}
