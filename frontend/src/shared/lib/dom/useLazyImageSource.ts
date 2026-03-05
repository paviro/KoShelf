import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';
import { useHasUserScrolled } from '../navigation/user-scroll-state';

type UseLazyImageSourceOptions = {
    src: string;
    rootMargin?: string;
    threshold?: number;
};

type UseLazyImageSourceResult = {
    imageRef: RefObject<HTMLImageElement | null>;
    resolvedSrc: string | undefined;
    isLoaded: boolean;
    hasError: boolean;
    shouldAnimateReveal: boolean;
    onLoad: () => void;
    onError: () => void;
};

export function useLazyImageSource({
    src,
    rootMargin = '50px 0px',
    threshold = 0.01,
}: UseLazyImageSourceOptions): UseLazyImageSourceResult {
    const imageRef = useRef<HTMLImageElement>(null);
    const [isIntersecting, setIsIntersecting] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);
    const [hasError, setHasError] = useState(false);
    const [shouldAnimateReveal, setShouldAnimateReveal] = useState(false);
    const hasResolvedRevealModeRef = useRef(false);
    const hasUserScrolled = useHasUserScrolled();
    const supportsIntersectionObserver =
        typeof window !== 'undefined' && 'IntersectionObserver' in window;

    const canLoadByIntersection = !supportsIntersectionObserver || isIntersecting;
    const shouldLoad = canLoadByIntersection;

    useEffect(() => {
        setIsIntersecting(false);
        setIsLoaded(false);
        setHasError(false);
        setShouldAnimateReveal(false);
        hasResolvedRevealModeRef.current = false;
    }, [src]);

    useEffect(() => {
        if (!shouldLoad || hasResolvedRevealModeRef.current) {
            return;
        }

        setShouldAnimateReveal(hasUserScrolled);
        hasResolvedRevealModeRef.current = true;
    }, [hasUserScrolled, shouldLoad]);

    useEffect(() => {
        if (typeof window === 'undefined') {
            return;
        }

        if (shouldLoad) {
            return;
        }

        const imageElement = imageRef.current;
        if (!imageElement) {
            return;
        }

        const observer = new IntersectionObserver(
            (entries) => {
                if (entries.some((entry) => entry.isIntersecting)) {
                    setIsIntersecting(true);
                    observer.disconnect();
                }
            },
            {
                rootMargin,
                threshold,
            },
        );

        observer.observe(imageElement);

        return () => {
            observer.disconnect();
        };
    }, [rootMargin, shouldLoad, threshold]);

    return {
        imageRef,
        resolvedSrc: shouldLoad ? src : undefined,
        isLoaded,
        hasError,
        shouldAnimateReveal,
        onLoad: () => setIsLoaded(true),
        onError: () => {
            setHasError(true);
            setIsLoaded(false);
        },
    };
}
