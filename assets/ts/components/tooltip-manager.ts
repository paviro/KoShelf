// Basic Tooltip Manager using custom JS positioning
export class TooltipManager {
    private static tooltipElement: HTMLElement | null = null;
    private static tooltipContentElement: HTMLElement | null = null;
    private static activeCell: HTMLElement | null = null;
    private static isVisible = false;
    private static hasGlobalListeners = false;

    static init(): void {
        if (!this.tooltipElement) {
            this.tooltipElement = document.createElement('div');
            // Basic visual styles (bg, text, rounding) - specialized positioning handled in CSS
            // NOTE: keep classes Tailwind-safe (Tailwind scans ./assets/ts/**/*.ts)
            this.tooltipElement.className =
                'heatmap-tooltip hidden z-50 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 text-xs px-2 py-1 rounded shadow-lg pointer-events-none whitespace-normal break-words opacity-90 [--tooltip-color:theme(colors.gray.900)] dark:[--tooltip-color:theme(colors.gray.100)]';
            this.tooltipElement.setAttribute('role', 'tooltip');

            // Dedicated content node (safer than innerHTML when setting content)
            this.tooltipContentElement = document.createElement('div');
            this.tooltipContentElement.className = 'heatmap-tooltip__content max-w-xs';
            this.tooltipElement.appendChild(this.tooltipContentElement);

            document.body.appendChild(this.tooltipElement);
        }

        if (!this.hasGlobalListeners) {
            this.hasGlobalListeners = true;

            // Close tooltip on click outside (for mobile)
            document.addEventListener('click', (e) => {
                const target = e.target instanceof Node ? e.target : null;
                if (this.activeCell && target && !this.activeCell.contains(target)) {
                    this.hide();
                }
            });

            // Reposition on scroll (capture catches scroll containers too)
            document.addEventListener(
                'scroll',
                () => {
                    if (this.activeCell && this.isVisible && this.tooltipElement) {
                        this.updatePosition(this.activeCell, this.tooltipElement);
                    }
                },
                { passive: true, capture: true },
            );

            // Reposition on resize
            window.addEventListener(
                'resize',
                () => {
                    if (this.activeCell && this.isVisible && this.tooltipElement) {
                        this.updatePosition(this.activeCell, this.tooltipElement);
                    }
                },
                { passive: true },
            );
        }
    }

    static attach(cell: HTMLElement, content: string): void {
        this.init(); // Ensure initialized

        // Prevent duplicate listeners if heatmap re-initializes
        if (cell.dataset.tooltipAttached === '1') return;
        cell.dataset.tooltipAttached = '1';

        const showTooltip = (): void => {
            // If clicking the same cell that is already active, toggle it off (optional, but good for mobile)
            if (this.activeCell === cell && !this.tooltipElement?.classList.contains('hidden')) {
                // On mobile/click, we might want to toggle, but for now we just show.
            }

            this.show(cell, content);
        };

        // Mouse enter (Desktop)
        cell.addEventListener('mouseenter', () => {
            showTooltip();
        });

        // Mouse leave (Desktop)
        cell.addEventListener('mouseleave', () => {
            this.hide();
        });

        // Click (Mobile/Desktop)
        cell.addEventListener('click', (e) => {
            e.stopPropagation(); // Prevent document click from immediately hiding it
            showTooltip();
        });
    }

    static show(cell: HTMLElement, content: string): void {
        if (!this.tooltipElement) return;

        this.activeCell = cell;
        if (this.tooltipContentElement) {
            this.tooltipContentElement.textContent = content;
        } else {
            this.tooltipElement.textContent = content;
        }

        // Always remove hidden class to ensure visibility (Tailwind .hidden overrides UA styles)
        this.tooltipElement.classList.remove('hidden');
        this.isVisible = true;

        // Calculate position
        this.updatePosition(cell, this.tooltipElement);
    }

    static hide(): void {
        if (this.tooltipElement) {
            this.tooltipElement.classList.add('hidden');
            this.isVisible = false;
            // Remove positioning classes
            this.tooltipElement.classList.remove(
                'tooltip-top',
                'tooltip-bottom',
                'tooltip-left',
                'tooltip-right',
            );
        }

        if (this.activeCell) {
            this.activeCell = null;
        }
    }

    // Calculate and apply position
    static updatePosition(cell: HTMLElement, tooltip: HTMLElement): void {
        if (!cell.isConnected) {
            this.hide();
            return;
        }

        const cellRect = cell.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;
        const padding = 8; // Padding from screen edges
        const gap = 8; // Gap between arrow tip and cell
        const arrowSize = 6; // Must match CSS border-width for the arrow

        // Ensure tooltip is measurable and constrained to viewport width
        tooltip.style.position = 'fixed';
        tooltip.style.maxWidth = `${Math.max(0, viewportWidth - padding * 2)}px`;

        // Measure tooltip size reliably (even if it was hidden before)
        const previousVisibility = tooltip.style.visibility;
        tooltip.style.visibility = 'hidden';
        tooltip.classList.remove('hidden');
        tooltip.style.top = '0px';
        tooltip.style.left = '0px';
        const tooltipRect = tooltip.getBoundingClientRect();
        tooltip.style.visibility = previousVisibility;

        const cellCenterX = cellRect.left + cellRect.width / 2;
        const cellCenterY = cellRect.top + cellRect.height / 2;

        const spaceTop = cellRect.top;
        const spaceBottom = viewportHeight - cellRect.bottom;
        const spaceLeft = cellRect.left;
        const spaceRight = viewportWidth - cellRect.right;

        type Placement = 'top' | 'bottom' | 'left' | 'right';
        const requiredV = tooltipRect.height + gap + arrowSize;
        const requiredH = tooltipRect.width + gap + arrowSize;

        const candidates: Array<{ placement: Placement; score: number }> = [
            {
                placement: 'top',
                score: spaceTop >= requiredV ? 10000 + spaceTop : spaceTop - requiredV,
            },
            {
                placement: 'bottom',
                score: spaceBottom >= requiredV ? 10000 + spaceBottom : spaceBottom - requiredV,
            },
            {
                placement: 'right',
                score: spaceRight >= requiredH ? 10000 + spaceRight : spaceRight - requiredH,
            },
            {
                placement: 'left',
                score: spaceLeft >= requiredH ? 10000 + spaceLeft : spaceLeft - requiredH,
            },
        ];
        candidates.sort((a, b) => b.score - a.score);
        const placement: Placement = candidates[0]?.placement ?? 'top';

        // Initial placement calculation
        let top = 0;
        let left = 0;

        if (placement === 'top') {
            top = cellRect.top - tooltipRect.height - gap - arrowSize;
            left = cellCenterX - tooltipRect.width / 2;
        } else if (placement === 'bottom') {
            top = cellRect.bottom + gap + arrowSize;
            left = cellCenterX - tooltipRect.width / 2;
        } else if (placement === 'right') {
            top = cellCenterY - tooltipRect.height / 2;
            left = cellRect.right + gap + arrowSize;
        } else {
            // left
            top = cellCenterY - tooltipRect.height / 2;
            left = cellRect.left - tooltipRect.width - gap - arrowSize;
        }

        // Clamp within viewport
        const maxLeft = Math.max(padding, viewportWidth - tooltipRect.width - padding);
        const maxTop = Math.max(padding, viewportHeight - tooltipRect.height - padding);
        left = Math.min(Math.max(left, padding), maxLeft);
        top = Math.min(Math.max(top, padding), maxTop);

        // Arrow offset relative to tooltip (clamped so it doesn't hit rounded corners)
        const arrowClampPadding = arrowSize + 6;
        const arrowX = Math.min(
            Math.max(cellCenterX - left, arrowClampPadding),
            Math.max(arrowClampPadding, tooltipRect.width - arrowClampPadding),
        );
        const arrowY = Math.min(
            Math.max(cellCenterY - top, arrowClampPadding),
            Math.max(arrowClampPadding, tooltipRect.height - arrowClampPadding),
        );

        // Apply styles
        tooltip.style.top = `${top}px`;
        tooltip.style.left = `${left}px`;
        tooltip.style.setProperty('--arrow-x', `${arrowX}px`);
        tooltip.style.setProperty('--arrow-y', `${arrowY}px`);

        // Set classes for arrow direction
        tooltip.classList.remove('tooltip-top', 'tooltip-bottom', 'tooltip-left', 'tooltip-right');
        tooltip.classList.add(`tooltip-${placement}`);
    }

    static cleanup(): void {
        if (this.tooltipElement) {
            this.tooltipElement.remove();
            this.tooltipElement = null;
            this.tooltipContentElement = null;
            this.isVisible = false;
        }
    }
}
