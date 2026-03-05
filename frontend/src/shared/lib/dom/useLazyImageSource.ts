import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';
import { useIsScrollRestoring } from '../navigation/scroll-restore-state';

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
    const isScrollRestoring = useIsScrollRestoring();

    const [prevSrc, setPrevSrc] = useState(src);
    if (prevSrc !== src) {
        setPrevSrc(src);
        setIsIntersecting(false);
        setIsLoaded(false);
        setHasError(false);
    }

    const canLoadByIntersection =
        !('IntersectionObserver' in window) || isIntersecting;
    const shouldLoad = !isScrollRestoring && canLoadByIntersection;

    useEffect(() => {
        if (shouldLoad || isScrollRestoring) {
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
    }, [isScrollRestoring, rootMargin, shouldLoad, threshold]);

    return {
        imageRef,
        resolvedSrc: shouldLoad ? src : undefined,
        isLoaded,
        hasError,
        onLoad: () => setIsLoaded(true),
        onError: () => {
            setHasError(true);
            setIsLoaded(false);
        },
    };
}
