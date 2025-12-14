// KoInsight - Lazy Loading Module

declare global {
    interface Window {
        LazyImageLoader: typeof LazyImageLoader;
    }
}

export class LazyImageLoader {
    private hasScrolled = false;
    private initialLoadProcessed = false;

    constructor() {
        this.setupScrollListener();
    }

    private setupScrollListener(): void {
        const scrollListener = () => {
            this.hasScrolled = true;
            window.removeEventListener('scroll', scrollListener);
        };
        window.addEventListener('scroll', scrollListener, { passive: true });
    }

    init(): void {
        const lazyImages = document.querySelectorAll<HTMLImageElement>('.lazy-image');

        // Progressive enhancement: Images already have src for no-JS fallback
        // For JS-enabled browsers, we'll optimize by lazy loading ALL images
        lazyImages.forEach((img) => {
            // Set all images to lazy load mode
            img.classList.add('opacity-0');
            // Clear src to prevent immediate loading, we'll load via data-src
            img.removeAttribute('src');
        });

        // Use Intersection Observer for lazy loading
        if ('IntersectionObserver' in window) {
            this.setupIntersectionObserver(lazyImages);
        } else {
            this.fallbackLoading(lazyImages);
        }
    }

    private setupIntersectionObserver(lazyImages: NodeListOf<HTMLImageElement>): void {
        const imageObserver = new IntersectionObserver((entries, observer) => {
            const intersectingEntries = entries.filter(entry => entry.isIntersecting);

            // Check if this is the initial load (images visible immediately on page load)
            const isInitialLoad = !this.hasScrolled && !this.initialLoadProcessed;

            if (isInitialLoad) {
                // Load initial images immediately without staggering
                intersectingEntries.forEach(entry => {
                    const img = entry.target as HTMLImageElement;
                    this.loadImageWithStagger(img); // No stagger delay
                    observer.unobserve(img);
                });
                this.initialLoadProcessed = true;
            } else {
                // Apply staggered loading for images that appear after scrolling
                const sortedEntries = intersectingEntries.sort((a, b) => {
                    const rectA = a.boundingClientRect;
                    const rectB = b.boundingClientRect;
                    // Sort by row first (top), then by column (left)
                    return rectA.top - rectB.top || rectA.left - rectB.left;
                });

                sortedEntries.forEach((entry, index) => {
                    const img = entry.target as HTMLImageElement;
                    const staggerDelay = index * 50; // 50ms between each image
                    this.loadImageWithStagger(img, staggerDelay);
                    observer.unobserve(img);
                });
            }
        }, {
            rootMargin: '50px 0px', // Start loading 50px before image comes into view
            threshold: 0.01
        });

        // Observe all images
        lazyImages.forEach((img) => {
            if (img.dataset.src) {
                imageObserver.observe(img);
            }
        });
    }

    private fallbackLoading(lazyImages: NodeListOf<HTMLImageElement>): void {
        // Fallback: load initial images immediately, rest with minimal stagger
        lazyImages.forEach((img, index) => {
            if (img.dataset.src) {
                if (index < 8) {
                    // Load first 8 images immediately
                    this.loadImageWithStagger(img);
                } else {
                    // Stagger the rest
                    const staggerDelay = (index - 8) * 100;
                    this.loadImageWithStagger(img, staggerDelay);
                }
            }
        });
    }

    private loadImageWithStagger(img: HTMLImageElement, staggerDelay = 0): void {
        if (!img.dataset.src || img.src) return; // Already loaded or no source

        const placeholder = img.nextElementSibling as HTMLElement | null;

        // Start loading the image
        img.src = img.dataset.src;
        img.onload = () => {
            // Apply stagger delay before showing the image
            setTimeout(() => {
                img.classList.remove('opacity-0');
                img.classList.add('opacity-100');

                // Fade out placeholder simultaneously
                if (placeholder?.classList.contains('book-placeholder')) {
                    placeholder.style.transition = 'opacity 0.3s ease-out';
                    placeholder.style.opacity = '0';

                    // Hide placeholder after fade completes
                    setTimeout(() => {
                        placeholder.style.display = 'none';
                    }, 300);
                }
            }, staggerDelay + 50); // Stagger delay + small delay to ensure image is ready
        };
        img.onerror = () => {
            // Keep placeholder visible on error, but still respect stagger timing
            setTimeout(() => {
                img.style.display = 'none';
                if (placeholder?.classList.contains('book-placeholder')) {
                    placeholder.style.display = 'flex';
                    placeholder.style.opacity = '1';
                }
            }, staggerDelay);
        };
    }

    // Method to load images for newly visible cards (used by filtering)
    loadImageForCard(card: HTMLElement): void {
        const lazyImage = card.querySelector<HTMLImageElement>('.lazy-image[data-src]');
        if (lazyImage && !lazyImage.src) {
            this.loadImageWithStagger(lazyImage);
        }
    }
}

// For non-module usage (script tag import)
window.LazyImageLoader = LazyImageLoader;
