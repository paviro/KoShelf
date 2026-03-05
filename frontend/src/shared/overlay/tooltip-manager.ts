import { AnchoredOverlay } from './anchored-overlay';

export class TooltipManager {
    private static overlay: AnchoredOverlay | null = null;
    private static highlightedElement: HTMLElement | null = null;

    private static applyHighlight(element: HTMLElement): void {
        element.classList.add(
            'ring-1',
            'ring-inset',
            'ring-gray-900',
            'dark:ring-white',
            'z-10',
        );
    }

    private static clearHighlight(element: HTMLElement): void {
        element.classList.remove(
            'ring-1',
            'ring-inset',
            'ring-gray-900',
            'dark:ring-white',
            'z-10',
        );
    }

    private static setHighlightedElement(element: HTMLElement | null): void {
        if (this.highlightedElement === element) {
            return;
        }

        if (this.highlightedElement) {
            this.clearHighlight(this.highlightedElement);
        }

        this.highlightedElement = element;

        if (this.highlightedElement) {
            this.applyHighlight(this.highlightedElement);
        }
    }

    private static init(): void {
        if (this.overlay) {
            return;
        }

        this.overlay = new AnchoredOverlay({
            className:
                'heatmap-tooltip z-50 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 text-xs px-2 py-1 rounded shadow-lg pointer-events-none whitespace-normal break-words opacity-90 [--tooltip-color:theme(colors.gray.900)] dark:[--tooltip-color:theme(colors.gray.100)]',
            contentClassName: 'heatmap-tooltip__content max-w-xs',
            hideClassName: 'hidden',
            placementClassPrefix: 'tooltip-',
            padding: 8,
            gap: 8,
            arrowSize: 6,
            maxWidthPadding: 8,
            hideOnOutsideClick: true,
            onVisibilityChange: (visible, anchor) => {
                this.setHighlightedElement(visible ? anchor : null);
            },
        });
    }

    static attach(element: HTMLElement, content: string): void {
        this.init();

        element.dataset.tooltipContent = content;
        if (element.dataset.tooltipAttached === '1') {
            return;
        }

        element.dataset.tooltipAttached = '1';

        const show = (): void => {
            const currentContent = element.dataset.tooltipContent || '';
            this.show(element, currentContent);
        };

        element.addEventListener('mouseenter', show);
        element.addEventListener('mouseleave', () => this.hide());
        element.addEventListener('click', (event) => {
            event.stopPropagation();
            show();
        });
    }

    static show(element: HTMLElement, content: string): void {
        this.init();

        const customGap = Number.parseFloat(element.dataset.tooltipGap || '');
        const gap = Number.isFinite(customGap) ? customGap : undefined;

        this.overlay?.show(
            element,
            content,
            gap !== undefined
                ? {
                      gap,
                  }
                : undefined,
        );
    }

    static hide(): void {
        this.overlay?.hide();
        this.setHighlightedElement(null);
    }
}
