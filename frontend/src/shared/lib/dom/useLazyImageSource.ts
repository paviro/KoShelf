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

type LazyImageState = {
    src: string;
    isIntersecting: boolean;
    isLoaded: boolean;
    hasError: boolean;
};

function createInitialState(src: string): LazyImageState {
    return {
        src,
        isIntersecting: false,
        isLoaded: false,
        hasError: false,
    };
}

export function useLazyImageSource({
    src,
    rootMargin = '50px 0px',
    threshold = 0.01,
}: UseLazyImageSourceOptions): UseLazyImageSourceResult {
    const imageRef = useRef<HTMLImageElement>(null);
    const [state, setState] = useState<LazyImageState>(() =>
        createInitialState(src),
    );
    const hasUserScrolled = useHasUserScrolled();
    const supportsIntersectionObserver =
        typeof window !== 'undefined' && 'IntersectionObserver' in window;
    const currentState = state.src === src ? state : createInitialState(src);

    const canLoadByIntersection =
        !supportsIntersectionObserver || currentState.isIntersecting;
    const shouldLoad = canLoadByIntersection;
    const shouldAnimateReveal = shouldLoad && hasUserScrolled;

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
                    setState((previous) => {
                        const current =
                            previous.src === src
                                ? previous
                                : createInitialState(src);
                        if (current.isIntersecting) {
                            return current;
                        }
                        return {
                            ...current,
                            isIntersecting: true,
                        };
                    });
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
    }, [rootMargin, shouldLoad, src, threshold]);

    return {
        imageRef,
        resolvedSrc: shouldLoad ? src : undefined,
        isLoaded: currentState.isLoaded,
        hasError: currentState.hasError,
        shouldAnimateReveal,
        onLoad: () => {
            setState((previous) => {
                const current =
                    previous.src === src ? previous : createInitialState(src);
                if (current.isLoaded && !current.hasError) {
                    return current;
                }
                return {
                    ...current,
                    isLoaded: true,
                    hasError: false,
                };
            });
        },
        onError: () => {
            setState((previous) => {
                const current =
                    previous.src === src ? previous : createInitialState(src);
                if (current.hasError && !current.isLoaded) {
                    return current;
                }
                return {
                    ...current,
                    hasError: true,
                    isLoaded: false,
                };
            });
        },
    };
}
