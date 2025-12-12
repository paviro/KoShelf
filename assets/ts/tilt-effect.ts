/**
 * 3D Tilt Effect Module
 * Creates subtle perspective rotation based on cursor position
 * Card tilts TOWARD the cursor (lifts toward it)
 */

export interface TiltOptions {
    /** CSS selector for elements to apply tilt effect */
    selector: string;
    /** Maximum rotation in degrees */
    maxRotation?: number;
    /** Transition duration in ms */
    transitionDuration?: number;
    /** Perspective depth in pixels */
    perspective?: number;
    /** Scale factor on hover */
    hoverScale?: number;
    /** Vertical lift on hover in px */
    hoverLift?: number;
    /** Shadow blur radius in px */
    shadowBlur?: number;
    /** Shadow opacity 0-1 */
    shadowOpacity?: number;
    /** Enable floating overlays parallax */
    enableOverlays?: boolean;
    /** Overlay float distance in px */
    overlayFloatZ?: number;
    /** Overlay scale factor */
    overlayScale?: number;
    /** Overlay parallax multiplier */
    parallaxMultiplier?: number;
    /** Selector for overlay container to enable 3D */
    overlayContainer?: string;
    /** Selector for badge overlays */
    badgeSelector?: string;
    /** Selector for progress bar */
    progressBarSelector?: string;
}

const DEFAULT_OPTIONS: Required<Omit<TiltOptions, 'selector'>> = {
    maxRotation: 4,
    transitionDuration: 150,
    perspective: 800,
    hoverScale: 1.02,
    hoverLift: 4,
    shadowBlur: 15,
    shadowOpacity: 0.20,
    enableOverlays: false,
    overlayFloatZ: 10,
    overlayScale: 1.05,
    parallaxMultiplier: 0.3,
    overlayContainer: '.aspect-book',
    badgeSelector: '[class*="absolute"][class*="top-"]',
    progressBarSelector: '.book-progress-bar',
};

/**
 * Initialize 3D tilt effect on elements matching the selector
 */
export function initTilt(options: TiltOptions): void {
    // Respect user's reduced motion preference
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        return;
    }

    const opts = { ...DEFAULT_OPTIONS, ...options };
    const elements = document.querySelectorAll<HTMLElement>(opts.selector);

    elements.forEach(element => {
        // Set up element for 3D transforms
        element.style.transformStyle = 'preserve-3d';
        element.style.transformOrigin = 'center center';
        element.style.willChange = 'transform';

        // Find overlays if enabled
        let overlayContainer: HTMLElement | null = null;
        let badgeOverlays: NodeListOf<HTMLElement> | null = null;
        let progressBar: HTMLElement | null = null;

        if (opts.enableOverlays && opts.overlayContainer) {
            overlayContainer = element.querySelector<HTMLElement>(opts.overlayContainer);
            if (overlayContainer) {
                overlayContainer.style.transformStyle = 'preserve-3d';
            }
            badgeOverlays = element.querySelectorAll<HTMLElement>(opts.badgeSelector);
            progressBar = element.querySelector<HTMLElement>(opts.progressBarSelector);

            // Prepare overlays
            badgeOverlays?.forEach(overlay => {
                overlay.style.willChange = 'transform';
            });
            if (progressBar) {
                progressBar.style.willChange = 'transform';
            }
        }

        element.addEventListener('mouseenter', () => {
            // Start transition for shadow fade-in (shadow will be applied on first mousemove)
            element.style.transition = `transform ${opts.transitionDuration}ms ease-out, filter ${opts.transitionDuration}ms ease-out`;
            // Apply initial subtle shadow that will transition to the dynamic one
            element.style.filter = `drop-shadow(0px 8px ${opts.shadowBlur}px rgba(0, 0, 0, ${opts.shadowOpacity * 0.5}))`;

            if (opts.enableOverlays && badgeOverlays) {
                badgeOverlays.forEach(overlay => {
                    overlay.style.transition = `transform ${opts.transitionDuration}ms ease-out`;
                    overlay.style.transform = `translateZ(${opts.overlayFloatZ}px) scale(${opts.overlayScale})`;
                });
            }
            if (opts.enableOverlays && progressBar) {
                progressBar.style.transition = `transform ${opts.transitionDuration}ms ease-out`;
                progressBar.style.transform = `translateZ(${opts.overlayFloatZ}px)`;
            }
        });

        element.addEventListener('mousemove', (e: MouseEvent) => {
            const rect = element.getBoundingClientRect();

            // Calculate cursor position relative to element center (0 to 1)
            const x = (e.clientX - rect.left) / rect.width;
            const y = (e.clientY - rect.top) / rect.height;

            // Calculate rotation - element LIFTS toward cursor
            const rotateY = (0.5 - x) * 2 * opts.maxRotation;
            const rotateX = (y - 0.5) * 2 * opts.maxRotation;

            // Calculate dynamic shadow offset
            const shadowX = -rotateY * 1.5;
            const shadowY = rotateX * 1.5 + 8;

            // Apply 3D rotation, scale, and lift
            element.style.transition = `transform ${opts.transitionDuration}ms ease-out, filter ${opts.transitionDuration}ms ease-out`;
            element.style.transform = `perspective(${opts.perspective}px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale(${opts.hoverScale}) translateY(-${opts.hoverLift}px)`;
            element.style.filter = `drop-shadow(${shadowX}px ${shadowY}px ${opts.shadowBlur}px rgba(0, 0, 0, ${opts.shadowOpacity}))`;

            // Apply parallax to overlays
            if (opts.enableOverlays) {
                const parallaxX = rotateY * opts.parallaxMultiplier;
                const parallaxY = -rotateX * opts.parallaxMultiplier;

                badgeOverlays?.forEach(overlay => {
                    overlay.style.transform = `translateZ(${opts.overlayFloatZ}px) translate(${parallaxX}px, ${parallaxY}px) scale(${opts.overlayScale})`;
                });

                if (progressBar) {
                    progressBar.style.transform = `translateZ(${opts.overlayFloatZ}px) translateX(${parallaxX}px)`;
                }
            }
        });

        element.addEventListener('mouseleave', () => {
            // Reset element transform and shadow
            element.style.transition = `transform ${opts.transitionDuration * 2}ms ease-out, filter ${opts.transitionDuration * 2}ms ease-out`;
            element.style.transform = '';
            element.style.filter = '';

            // Reset overlays
            if (opts.enableOverlays) {
                badgeOverlays?.forEach(overlay => {
                    overlay.style.transition = `transform ${opts.transitionDuration * 2}ms ease-out`;
                    overlay.style.transform = '';
                });

                if (progressBar) {
                    progressBar.style.transition = `transform ${opts.transitionDuration * 2}ms ease-out`;
                    progressBar.style.transform = '';
                }
            }
        });
    });
}

/**
 * Pre-configured tilt for book cards (main book list)
 */
export function initBookCardTilt(): void {
    initTilt({
        selector: '.book-card',
        enableOverlays: true,
    });
}

/**
 * Pre-configured tilt for recap covers (lighter effect)
 */
export function initRecapCoverTilt(): void {
    initTilt({
        selector: '.recap-cover-tilt',
        maxRotation: 3,
        hoverScale: 1.02,
        hoverLift: 3,
        shadowBlur: 20,
        shadowOpacity: 0.15,
        enableOverlays: false,
    });
}
