import {
    useCallback,
    useEffect,
    useRef,
    useState,
    type PointerEvent as ReactPointerEvent,
    type RefObject,
} from 'react';

import type { FoliateView } from '../model/reader-model';

type UseReaderScrubberResult = {
    trackRef: RefObject<HTMLDivElement | null>;
    dragging: boolean;
    dragFraction: number | null;
    scrubSettlingRef: RefObject<boolean>;
    setDragFraction: (value: number | null) => void;
    handleScrubStart: (e: ReactPointerEvent<HTMLDivElement>) => void;
    handleScrubMove: (e: ReactPointerEvent<HTMLDivElement>) => void;
    handleScrubEnd: (e: ReactPointerEvent<HTMLDivElement>) => void;
};

export function useReaderScrubber(
    viewRef: RefObject<FoliateView | null>,
): UseReaderScrubberResult {
    const trackRef = useRef<HTMLDivElement>(null);
    const [dragging, setDragging] = useState(false);
    const [dragFraction, setDragFraction] = useState<number | null>(null);
    const scrubSettlingRef = useRef(false);
    const scrubTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const pendingScrubRef = useRef<number | null>(null);

    useEffect(
        () => () => {
            if (scrubTimerRef.current !== null) {
                clearTimeout(scrubTimerRef.current);
                scrubTimerRef.current = null;
            }
            pendingScrubRef.current = null;
        },
        [],
    );

    const fractionFromPointer = useCallback((clientX: number) => {
        const track = trackRef.current;
        if (!track) {
            return 0;
        }

        const rect = track.getBoundingClientRect();
        if (rect.width <= 0) {
            return 0;
        }

        return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    }, []);

    const navigateThrottled = useCallback(
        (frac: number) => {
            pendingScrubRef.current = frac;
            if (scrubTimerRef.current !== null) {
                return;
            }

            scrubTimerRef.current = setTimeout(() => {
                scrubTimerRef.current = null;
                const pending = pendingScrubRef.current;
                if (pending !== null) {
                    pendingScrubRef.current = null;
                    void viewRef.current?.goToFraction(pending);
                }
            }, 150);
        },
        [viewRef],
    );

    const handleScrubStart = useCallback(
        (e: ReactPointerEvent<HTMLDivElement>) => {
            e.preventDefault();
            e.currentTarget.setPointerCapture(e.pointerId);
            setDragging(true);
            const frac = fractionFromPointer(e.clientX);
            setDragFraction(frac);
            navigateThrottled(frac);
        },
        [fractionFromPointer, navigateThrottled],
    );

    const handleScrubMove = useCallback(
        (e: ReactPointerEvent<HTMLDivElement>) => {
            if (!dragging) return;
            const frac = fractionFromPointer(e.clientX);
            setDragFraction(frac);
            navigateThrottled(frac);
        },
        [dragging, fractionFromPointer, navigateThrottled],
    );

    const handleScrubEnd = useCallback(
        (e: ReactPointerEvent<HTMLDivElement>) => {
            if (!dragging) {
                return;
            }

            setDragging(false);

            if (e.currentTarget.hasPointerCapture(e.pointerId)) {
                e.currentTarget.releasePointerCapture(e.pointerId);
            }

            if (scrubTimerRef.current !== null) {
                clearTimeout(scrubTimerRef.current);
                scrubTimerRef.current = null;
            }

            pendingScrubRef.current = null;
            const frac = fractionFromPointer(e.clientX);
            setDragFraction(frac);
            scrubSettlingRef.current = true;
            void viewRef.current?.goToFraction(frac);
        },
        [dragging, fractionFromPointer, viewRef],
    );

    return {
        trackRef,
        dragging,
        dragFraction,
        scrubSettlingRef,
        setDragFraction,
        handleScrubStart,
        handleScrubMove,
        handleScrubEnd,
    };
}
