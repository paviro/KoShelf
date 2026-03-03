/**
 * KoReader Activity Heatmap Module
 * Handles loading and displaying reading activity heatmap with year selection
 */

import { translation } from '../shared/i18n.js';
import { TooltipManager } from './tooltip-manager.js';
import { setActiveOption } from '../shared/active-option.js';
import {
    loadYearlyActivity,
    type ActivityConfig,
    type DailyActivityEntry,
} from '../shared/statistics-data-loader.js';

interface ActivityData {
    pages: number;
    read: number;
}

const HEATMAP_COLOR_CLASSES = [
    ['bg-gray-100', 'dark:bg-dark-800'],
    ['bg-green-100', 'dark:bg-green-900'],
    ['bg-green-300', 'dark:bg-green-700'],
    ['bg-green-500', 'dark:bg-green-500'],
    ['bg-green-600', 'dark:bg-green-300'],
] as const;

const HEATMAP_ALL_COLOR_CLASSES = HEATMAP_COLOR_CLASSES.flat();

const HEATMAP_YEAR_SELECTOR_CLASS_STATE = {
    active: ['bg-dark-700', 'text-white'],
    inactive: ['text-dark-200'],
} as const;

class ActivityHeatmap {
    private activityData: DailyActivityEntry[] | null = null;
    private activityConfig: ActivityConfig | null = null;
    private availableYears: number[] = [];
    private currentYear: number | null = null;
    private loadRequestId = 0;
    private isInitialized = false;
    private resizeObserver: ResizeObserver | null = null;
    private basePath = '/assets/json/statistics';
    private shouldAnimateYearTransition = false;
    private cellAnimationTimeoutIds: number[] = [];

    // Initialize the heatmap module
    async init(): Promise<void> {
        if (this.isInitialized) return;

        try {
            // Load translations
            await translation.init();

            // JSON base path comes from server-rendered page scope
            const base = document.body.getAttribute('data-stats-json-base');
            if (base) this.basePath = base;

            // Get available years from the template-rendered year options
            this.getAvailableYearsFromTemplate();

            // Initialize year selector
            this.initializeYearSelector();

            // Load data for the current year (most recent by default)
            if (this.availableYears.length > 0) {
                this.currentYear = this.availableYears[0]; // Most recent year first
                await this.loadYearData(this.currentYear);
                this.shouldAnimateYearTransition = !this.isBackForwardNavigation();
                this.initializeHeatmap();
            }

            this.isInitialized = true;
        } catch (error) {
            console.error('Error initializing heatmap:', error);
        }
    }

    // Get available years from the template-rendered year options
    private getAvailableYearsFromTemplate(): void {
        const yearOptions = document.querySelectorAll<HTMLElement>('.year-option');
        this.availableYears = Array.from(yearOptions)
            .map((option) => parseInt(option.getAttribute('data-year') || '0'))
            .filter((year) => year > 0);
    }

    // Load activity data for a specific year
    private async loadYearData(year: number): Promise<void> {
        const currentLoadRequestId = ++this.loadRequestId;

        try {
            const yearlyActivity = await loadYearlyActivity(this.basePath, year);

            if (currentLoadRequestId !== this.loadRequestId) return;

            this.activityData = yearlyActivity.data;
            this.activityConfig = yearlyActivity.config ?? { max_scale_seconds: null };

            this.currentYear = year;
        } catch (error) {
            if (currentLoadRequestId !== this.loadRequestId) return;

            console.error(`Error loading activity data for ${year}:`, error);
            this.activityData = [];
            this.activityConfig = { max_scale_seconds: null };
            this.currentYear = year;
        }
    }

    // (No filtering here; years are already scoped by the server-rendered page.)

    // Initialize year selector functionality
    private initializeYearSelector(): void {
        const yearSelectorWrapper = document.getElementById('yearSelectorWrapper');
        const yearOptions = document.getElementById('yearOptions');

        if (!yearSelectorWrapper || !yearOptions) return;

        const setYearSelectorExpanded = (expanded: boolean): void => {
            yearSelectorWrapper.setAttribute('aria-expanded', String(expanded));
        };

        // Add click handlers to existing year options
        this.setupYearOptionHandlers();

        // Mark the first year option as selected initially
        const firstYearOption = document.querySelector<HTMLElement>('.year-option');
        if (firstYearOption) {
            this.updateActiveYearOption(firstYearOption);
        }

        // Handle dropdown toggle
        yearSelectorWrapper.addEventListener('click', () => {
            yearOptions.classList.toggle('hidden');
            setYearSelectorExpanded(!yearOptions.classList.contains('hidden'));
        });
    }

    // Setup click handlers for existing year options
    private setupYearOptionHandlers(): void {
        const yearOptionElements = document.querySelectorAll<HTMLElement>('.year-option');

        yearOptionElements.forEach((option) => {
            // Add click handler
            option.addEventListener('click', async () => {
                const selectedYear = parseInt(option.getAttribute('data-year') || '0');
                if (selectedYear > 0) {
                    await this.selectYear(selectedYear);

                    // Update UI
                    this.updateActiveYearOption(option);
                    this.updateSelectedYearText(selectedYear);

                    // Close dropdown
                    document.getElementById('yearOptions')?.classList.add('hidden');
                    document
                        .getElementById('yearSelectorWrapper')
                        ?.setAttribute('aria-expanded', 'false');
                }
            });
        });
    }

    // Select a specific year and reload heatmap
    private async selectYear(year: number): Promise<void> {
        if (year === this.currentYear) return;

        try {
            this.shouldAnimateYearTransition = this.currentYear !== null;

            // Load new year data
            await this.loadYearData(year);

            // Reinitialize heatmap with new data
            this.initializeHeatmap();
        } catch (error) {
            console.error(`Error selecting year ${year}:`, error);
        }
    }

    // Update active year option in dropdown
    private updateActiveYearOption(selectedOption: HTMLElement): void {
        const allOptions = document.querySelectorAll<HTMLElement>('.year-option');
        setActiveOption(allOptions, selectedOption, HEATMAP_YEAR_SELECTOR_CLASS_STATE);
    }

    // Update selected year text
    private updateSelectedYearText(year: number): void {
        const selectedYearText = document.getElementById('selectedYearText');
        if (selectedYearText) {
            selectedYearText.innerHTML = `<span class="font-bold">${year}</span>`;
        }
    }

    // Initialize and render the heatmap
    private initializeHeatmap(): void {
        if (!this.activityData || this.currentYear === null) return;

        // Setup height synchronization
        this.setupHeightSync();

        // Process data and create activity map
        const { activityMap, maxActivity } = this.processActivityData(this.activityData);

        // Fill the heatmap cells
        this.fillHeatmapCells(activityMap, maxActivity);

        // Auto-scroll to show current month for current year
        if (this.currentYear === new Date().getFullYear()) {
            this.scrollToCurrentMonth();
        } else {
            // For past years, scroll to show the end of the year
            this.scrollToEndOfYear();
        }

        // Setup resize observer for height sync
        this.setupResizeObserver();
    }

    // Setup height synchronization between day labels and heatmap grid
    private setupHeightSync(): void {
        const heatmapGrid = document.getElementById('heatmapGrid');
        const dayLabels = document.getElementById('dayLabels');

        if (heatmapGrid && dayLabels) {
            dayLabels.style.height = heatmapGrid.offsetHeight + 'px';
        }
    }

    // Process activity data and find maximum activity level
    private processActivityData(activityData: DailyActivityEntry[]): {
        activityMap: Map<string, ActivityData>;
        maxActivity: number;
    } {
        const activityMap = new Map<string, ActivityData>();
        let maxActivity = 0;

        // Find max reading time and fill map
        activityData.forEach((day) => {
            if (day.read_time > maxActivity) {
                maxActivity = day.read_time;
            }
            activityMap.set(day.date, {
                pages: day.pages_read,
                read: day.read_time,
            });
        });

        // Use custom max scale if provided
        if (
            this.activityConfig?.max_scale_seconds !== null &&
            this.activityConfig?.max_scale_seconds !== undefined
        ) {
            maxActivity = this.activityConfig.max_scale_seconds;
        }

        return { activityMap, maxActivity };
    }

    // Fill heatmap cells with activity data
    private fillHeatmapCells(activityMap: Map<string, ActivityData>, maxActivity: number): void {
        const cells = document.querySelectorAll<HTMLElement>('.activity-cell');
        const shouldAnimate = this.shouldAnimateYearTransition && !this.prefersReducedMotion();

        this.clearPendingCellPopIns();

        cells.forEach((cell) => {
            if (this.currentYear === null) return;

            // Calculate the date for this cell
            const cellDate = this.calculateCellDate(cell, this.currentYear);
            const dateStr = DateUtils.formatDateAsISO(cellDate);

            // Get activity level for this date (use reading time)
            const activityObj = activityMap.get(dateStr) || { pages: 0, read: 0 };
            const activity = activityObj.read;

            // Normalize and apply activity level
            const activityLevel = this.normalizeActivityLevel(activity, maxActivity);
            const shouldAnimateCell = shouldAnimate && activityLevel > 0;
            const animationDelay = shouldAnimateCell ? this.calculateCellAnimationDelay() : 0;
            this.applyCellStyling(
                cell,
                activityLevel,
                dateStr,
                activityObj,
                shouldAnimateCell,
                animationDelay,
            );
        });

        this.shouldAnimateYearTransition = false;
    }

    // Calculate the date represented by a heatmap cell for a specific year
    private calculateCellDate(cell: HTMLElement, year: number): Date {
        const weekIndex = parseInt(cell.getAttribute('data-week') || '0');
        const dayIndex = parseInt(cell.getAttribute('data-day') || '0');

        // Compute the date based on the first Monday of the specified year
        const janFirst = new Date(year, 0, 1);

        // Find the Monday on (or before) Jan 1
        const janDayOfWeek = janFirst.getDay();
        const shiftToMonday = janDayOfWeek === 0 ? -6 : 1 - janDayOfWeek;
        const firstMonday = new Date(janFirst);
        firstMonday.setDate(janFirst.getDate() + shiftToMonday);

        // Now compute the cell's date
        const cellDate = new Date(firstMonday);
        cellDate.setDate(cellDate.getDate() + weekIndex * 7 + dayIndex);

        return cellDate;
    }

    // Normalize activity level to 0-4 range
    private normalizeActivityLevel(activity: number, maxActivity: number): number {
        let activityLevel = 0;
        if (activity > 0) {
            if (maxActivity <= 4) {
                activityLevel = activity;
            } else {
                activityLevel = Math.min(4, Math.ceil((activity / maxActivity) * 4));
            }
        }
        return activityLevel;
    }

    // Apply styling and interactions to a heatmap cell
    private applyCellStyling(
        cell: HTMLElement,
        activityLevel: number,
        dateStr: string,
        activityObj: ActivityData,
        shouldAnimate = false,
        animationDelay = 0,
    ): void {
        cell.getAnimations().forEach((animation) => animation.cancel());
        HEATMAP_ALL_COLOR_CLASSES.forEach((cls) => cell.classList.remove(cls));

        if (shouldAnimate) {
            HEATMAP_COLOR_CLASSES[0].forEach((cls) => cell.classList.add(cls));

            const timeoutId = window.setTimeout(() => {
                HEATMAP_COLOR_CLASSES[0].forEach((cls) => cell.classList.remove(cls));
                HEATMAP_COLOR_CLASSES[activityLevel].forEach((cls) => cell.classList.add(cls));
                this.applyCellTransitionAnimation(cell);
            }, animationDelay);

            this.cellAnimationTimeoutIds.push(timeoutId);
        } else {
            HEATMAP_COLOR_CLASSES[activityLevel].forEach((cls) => cell.classList.add(cls));
        }

        // Prepare tooltip content
        const readLabel = DateUtils.formatDuration(activityObj.read);
        const tooltipContent = `${dateStr}: ${readLabel}, ${activityObj.pages} ${translation.get('pages-label', activityObj.pages)}`;

        // Use custom tooltip manager instead of title attribute
        TooltipManager.attach(cell, tooltipContent);

        // Add hover functionality for highlighting
        this.addCellHoverEffects(cell);
    }

    private clearPendingCellPopIns(): void {
        this.cellAnimationTimeoutIds.forEach((timeoutId) => {
            window.clearTimeout(timeoutId);
        });
        this.cellAnimationTimeoutIds = [];
    }

    private applyCellTransitionAnimation(cell: HTMLElement): void {
        if (typeof cell.animate !== 'function') {
            return;
        }

        cell.getAnimations().forEach((animation) => animation.cancel());

        cell.animate(
            [
                { transform: 'scale(0.62)', filter: 'saturate(0.88) brightness(0.99)' },
                {
                    transform: 'scale(1.2)',
                    filter: 'saturate(1.22) brightness(1.04)',
                    offset: 0.52,
                },
                {
                    transform: 'scale(0.97)',
                    filter: 'saturate(1.03) brightness(1.01)',
                    offset: 0.78,
                },
                { transform: 'scale(1)', filter: 'saturate(1) brightness(1)' },
            ],
            {
                duration: 280,
                easing: 'cubic-bezier(0.2, 0.9, 0.24, 1.1)',
                fill: 'both',
            },
        );
    }

    private calculateCellAnimationDelay(): number {
        return Math.floor(Math.random() * 340);
    }

    private prefersReducedMotion(): boolean {
        return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    }

    private isBackForwardNavigation(): boolean {
        const navigationEntry = performance.getEntriesByType('navigation')[0] as
            | PerformanceNavigationTiming
            | undefined;

        if (navigationEntry) {
            return navigationEntry.type === 'back_forward';
        }

        return (
            typeof performance !== 'undefined' &&
            'navigation' in performance &&
            performance.navigation.type === 2
        );
    }

    // Add hover effects to a cell
    private addCellHoverEffects(cell: HTMLElement): void {
        if (cell.dataset.hoverEffectsBound === 'true') return;
        cell.dataset.hoverEffectsBound = 'true';

        cell.addEventListener('mouseover', function (this: HTMLElement) {
            this.classList.add('ring-1', 'ring-inset', 'ring-gray-900', 'dark:ring-white', 'z-10');
        });

        cell.addEventListener('mouseout', function (this: HTMLElement) {
            this.classList.remove(
                'ring-1',
                'ring-inset',
                'ring-gray-900',
                'dark:ring-white',
                'z-10',
            );
        });
    }

    // Auto-scroll to show current month positioned towards the right
    private scrollToCurrentMonth(): void {
        const scrollContainer = document.getElementById('heatmapScrollContainer');
        const heatmapContainer = document.getElementById('readingHeatmap');

        if (!scrollContainer || !heatmapContainer) return;

        // Calculate current week position
        const today = new Date();
        const currentWeek = this.calculateCurrentWeek(today);

        // Get container dimensions
        const containerWidth = scrollContainer.clientWidth;
        const heatmapWidth = heatmapContainer.scrollWidth;

        // Only scroll if content overflows
        if (heatmapWidth > containerWidth) {
            // Calculate week width (approximate)
            const weekWidth = heatmapWidth / 53;

            // Position current week at 70% from the left (towards the right)
            const targetPosition = currentWeek * weekWidth - containerWidth * 0.7;

            // Ensure we don't scroll past the beginning or end
            const maxScroll = heatmapWidth - containerWidth;
            const scrollPosition = Math.max(0, Math.min(targetPosition, maxScroll));

            scrollContainer.scrollLeft = scrollPosition;
        }
    }

    // Scroll to end of year for past years
    private scrollToEndOfYear(): void {
        const scrollContainer = document.getElementById('heatmapScrollContainer');
        const heatmapContainer = document.getElementById('readingHeatmap');

        if (!scrollContainer || !heatmapContainer) return;

        // Get container dimensions
        const containerWidth = scrollContainer.clientWidth;
        const heatmapWidth = heatmapContainer.scrollWidth;

        // Only scroll if content overflows
        if (heatmapWidth > containerWidth) {
            // Scroll to show the end of the year (rightmost part)
            const maxScroll = heatmapWidth - containerWidth;
            scrollContainer.scrollLeft = maxScroll * 0.8; // Show 80% towards the end
        }
    }

    // Calculate which week the current date falls into
    private calculateCurrentWeek(today: Date): number {
        const baseYear = today.getFullYear();
        const janFirst = new Date(baseYear, 0, 1);

        // Find the Monday on (or before) Jan 1
        const janDayOfWeek = janFirst.getDay();
        const shiftToMonday = janDayOfWeek === 0 ? -6 : 1 - janDayOfWeek;
        const firstMonday = new Date(janFirst);
        firstMonday.setDate(janFirst.getDate() + shiftToMonday);

        // Calculate which week the current date falls into
        const daysDiff = Math.floor(
            (today.getTime() - firstMonday.getTime()) / (1000 * 60 * 60 * 24),
        );
        return Math.floor(daysDiff / 7);
    }

    // Setup resize observer to keep heights in sync
    private setupResizeObserver(): void {
        if (this.resizeObserver) {
            this.resizeObserver.disconnect();
        }

        this.resizeObserver = new ResizeObserver(() => {
            this.setupHeightSync();
        });

        const heatmapGrid = document.getElementById('heatmapGrid');
        if (heatmapGrid) {
            this.resizeObserver.observe(heatmapGrid);
        }
    }

    // Cleanup method
    destroy(): void {
        this.clearPendingCellPopIns();

        if (this.resizeObserver) {
            this.resizeObserver.disconnect();
            this.resizeObserver = null;
        }
        this.isInitialized = false;
        TooltipManager.cleanup();
    }
}

// Date utility functions
class DateUtils {
    // Format date as YYYY-MM-DD for lookup
    static formatDateAsISO(date: Date): string {
        return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
    }

    // Format seconds into "Xh Ym" style
    static formatDuration(secs: number): string {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const parts: string[] = [];
        if (h) parts.push(`${h}h`);
        if (m || !h) parts.push(`${m}m`);
        return parts.join(' ');
    }
}

// Create and export the heatmap instance
const activityHeatmap = new ActivityHeatmap();

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => activityHeatmap.init());
} else {
    activityHeatmap.init();
}

// Export for module use
export { activityHeatmap as default, ActivityHeatmap, DateUtils };
