import { useEffect } from 'react';
import type { DependencyList } from 'react';

export interface TiltOptions {
    selector: string;
    maxRotation?: number;
    transitionDuration?: number;
    perspective?: number;
    hoverScale?: number;
    hoverLift?: number;
    shadowBlur?: number;
    shadowOpacity?: number;
    enableOverlays?: boolean;
    overlayFloatZ?: number;
    overlayScale?: number;
    parallaxMultiplier?: number;
    overlayContainer?: string;
    badgeSelector?: string;
    progressBarSelector?: string;
}

const DEFAULT_OPTIONS: Required<Omit<TiltOptions, 'selector'>> = {
    maxRotation: 4,
    transitionDuration: 150,
    perspective: 800,
    hoverScale: 1.02,
    hoverLift: 4,
    shadowBlur: 15,
    shadowOpacity: 0.2,
    enableOverlays: false,
    overlayFloatZ: 10,
    overlayScale: 1.05,
    parallaxMultiplier: 0.3,
    overlayContainer: '.aspect-book',
    badgeSelector: '[class*="absolute"][class*="top-"]',
    progressBarSelector: '.book-progress-bar',
};

type Cleanup = () => void;

function resetCardStyles(element: HTMLElement): void {
    element.style.transformStyle = '';
    element.style.transformOrigin = '';
    element.style.willChange = '';
    element.style.backfaceVisibility = '';
    element.style.transform = '';
    element.style.transition = '';
    element.style.boxShadow = '';
}

function resetOverlayStyles(overlays: HTMLElement[]): void {
    overlays.forEach((overlay) => {
        overlay.style.willChange = '';
        overlay.style.backfaceVisibility = '';
        overlay.style.transform = '';
        overlay.style.transition = '';
    });
}

function initTilt(options: TiltOptions): Cleanup {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        return () => {};
    }

    if (!window.matchMedia('(pointer: fine)').matches) {
        return () => {};
    }

    const opts = { ...DEFAULT_OPTIONS, ...options };
    const elements = Array.from(document.querySelectorAll<HTMLElement>(opts.selector));
    const cleanupHandlers: Cleanup[] = [];

    elements.forEach((element) => {
        element.style.transformStyle = 'preserve-3d';
        element.style.transformOrigin = 'center center';
        element.style.willChange = 'transform, box-shadow';
        element.style.backfaceVisibility = 'hidden';
        element.style.transform = 'translateZ(0)';

        let overlayContainer: HTMLElement | null = null;
        const overlayElements: HTMLElement[] = [];

        if (opts.enableOverlays) {
            overlayContainer = element.querySelector<HTMLElement>(opts.overlayContainer);
            if (overlayContainer) {
                overlayContainer.style.transformStyle = 'preserve-3d';
                overlayContainer.style.backfaceVisibility = 'hidden';
            }

            const badges = Array.from(element.querySelectorAll<HTMLElement>(opts.badgeSelector));
            overlayElements.push(...badges);

            const progressBar = element.querySelector<HTMLElement>(opts.progressBarSelector);
            if (progressBar) {
                overlayElements.push(progressBar);
            }

            overlayElements.forEach((overlay) => {
                overlay.style.willChange = 'transform';
                overlay.style.backfaceVisibility = 'hidden';
                overlay.style.transform = 'translateZ(0)';
            });
        }

        let rafId: number | null = null;
        let pendingMouseEvent: MouseEvent | null = null;

        const handleMouseEnter = (): void => {
            element.style.transition = `transform ${opts.transitionDuration}ms ease-out, box-shadow ${opts.transitionDuration}ms ease-out`;
            element.style.boxShadow = `0px 8px ${opts.shadowBlur}px rgba(0, 0, 0, ${opts.shadowOpacity * 0.5})`;

            if (!opts.enableOverlays) {
                return;
            }

            overlayElements.forEach((overlay) => {
                overlay.style.transition = `transform ${opts.transitionDuration}ms ease-out`;
                if (overlay.matches(opts.progressBarSelector)) {
                    overlay.style.transform = `translateZ(${opts.overlayFloatZ}px)`;
                    return;
                }

                overlay.style.transform = `translateZ(${opts.overlayFloatZ}px) scale(${opts.overlayScale})`;
            });
        };

        const updateTransform = (): void => {
            if (!pendingMouseEvent) {
                rafId = null;
                return;
            }

            const event = pendingMouseEvent;
            const rect = element.getBoundingClientRect();

            const x = (event.clientX - rect.left) / rect.width;
            const y = (event.clientY - rect.top) / rect.height;

            const rotateY = (0.5 - x) * 2 * opts.maxRotation;
            const rotateX = (y - 0.5) * 2 * opts.maxRotation;

            const shadowX = -rotateY * 1.5;
            const shadowY = rotateX * 1.5 + 8;

            element.style.transition = `transform ${opts.transitionDuration}ms ease-out, box-shadow ${opts.transitionDuration}ms ease-out`;
            element.style.transform = `perspective(${opts.perspective}px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale(${opts.hoverScale}) translateY(-${opts.hoverLift}px) translateZ(0)`;
            element.style.boxShadow = `${shadowX}px ${shadowY}px ${opts.shadowBlur}px rgba(0, 0, 0, ${opts.shadowOpacity})`;

            if (opts.enableOverlays) {
                const parallaxX = rotateY * opts.parallaxMultiplier;
                const parallaxY = -rotateX * opts.parallaxMultiplier;

                overlayElements.forEach((overlay) => {
                    if (overlay.matches(opts.progressBarSelector)) {
                        overlay.style.transform = `translateZ(${opts.overlayFloatZ}px) translateX(${parallaxX}px)`;
                        return;
                    }

                    if (overlay.matches(opts.badgeSelector)) {
                        overlay.style.transform = `translateZ(${opts.overlayFloatZ}px) translate(${parallaxX}px, ${parallaxY}px) scale(${opts.overlayScale})`;
                    }
                });
            }

            pendingMouseEvent = null;
            rafId = null;
        };

        const handleMouseMove = (event: MouseEvent): void => {
            pendingMouseEvent = event;
            if (rafId !== null) {
                return;
            }

            rafId = window.requestAnimationFrame(updateTransform);
        };

        const handleMouseLeave = (): void => {
            if (rafId !== null) {
                window.cancelAnimationFrame(rafId);
                rafId = null;
            }

            pendingMouseEvent = null;

            element.style.transition = `transform ${opts.transitionDuration * 2}ms ease-out, box-shadow ${opts.transitionDuration * 2}ms ease-out`;
            element.style.transform = 'translateZ(0)';
            element.style.boxShadow = '';

            if (!opts.enableOverlays) {
                return;
            }

            overlayElements.forEach((overlay) => {
                overlay.style.transition = `transform ${opts.transitionDuration * 2}ms ease-out`;
                overlay.style.transform = 'translateZ(0)';
            });
        };

        element.addEventListener('mouseenter', handleMouseEnter);
        element.addEventListener('mousemove', handleMouseMove);
        element.addEventListener('mouseleave', handleMouseLeave);

        cleanupHandlers.push(() => {
            if (rafId !== null) {
                window.cancelAnimationFrame(rafId);
                rafId = null;
            }

            element.removeEventListener('mouseenter', handleMouseEnter);
            element.removeEventListener('mousemove', handleMouseMove);
            element.removeEventListener('mouseleave', handleMouseLeave);

            resetCardStyles(element);
            resetOverlayStyles(overlayElements);

            if (overlayContainer) {
                overlayContainer.style.transformStyle = '';
                overlayContainer.style.backfaceVisibility = '';
            }
        });
    });

    return () => {
        cleanupHandlers.forEach((cleanup) => cleanup());
    };
}

export function useTiltEffect(options: TiltOptions, dependencies: DependencyList): void {
    useEffect(() => {
        const teardownTilt = initTilt(options);
        return () => {
            teardownTilt();
        };
    }, [options, ...dependencies]);
}

export function useBookCardTiltEffect(dependencies: DependencyList): void {
    useEffect(() => {
        const teardownTilt = initTilt({
            selector: '.book-card',
            enableOverlays: true,
        });
        return () => {
            teardownTilt();
        };
    }, dependencies);
}

export function useRecapCoverTiltEffect(dependencies: DependencyList): void {
    useEffect(() => {
        const teardownTilt = initTilt({
            selector: '.recap-cover-tilt',
            maxRotation: 3,
            hoverScale: 1.02,
            hoverLift: 3,
            shadowBlur: 20,
            shadowOpacity: 0.15,
            enableOverlays: false,
        });
        return () => {
            teardownTilt();
        };
    }, dependencies);
}
