import { useEffect, useRef, useState } from 'react';
import type { RefObject } from 'react';

type UseLazyImageSourceOptions = {
    src: string;
    rootMargin?: string;
    threshold?: number;
};

type UseLazyImageSourceResult = {
    imageRef: RefObject<HTMLImageElement>;
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
    const [shouldLoad, setShouldLoad] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);
    const [hasError, setHasError] = useState(false);

    useEffect(() => {
        setShouldLoad(false);
        setIsLoaded(false);
        setHasError(false);
    }, [src]);

    useEffect(() => {
        if (shouldLoad) {
            return;
        }

        const imageElement = imageRef.current;
        if (!imageElement) {
            return;
        }

        if (!('IntersectionObserver' in window)) {
            setShouldLoad(true);
            return;
        }

        const observer = new IntersectionObserver(
            (entries) => {
                const isIntersecting = entries.some((entry) => entry.isIntersecting);
                if (!isIntersecting) {
                    return;
                }

                setShouldLoad(true);
                observer.disconnect();
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
        onLoad: () => setIsLoaded(true),
        onError: () => {
            setHasError(true);
            setIsLoaded(false);
        },
    };
}
