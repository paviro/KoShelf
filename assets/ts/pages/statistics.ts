/**
 * KoReader Statistics Viewer Module
 * Handles loading and displaying reading statistics
 */

import { translation } from '../shared/i18n.js';
import { SectionToggle } from '../components/section-toggle.js';
import { DateFormatter, DataFormatter } from '../shared/statistics-formatters.js';
import { setActiveOption } from '../shared/active-option.js';
import { YearlyStatsChart } from '../components/yearly-stats-chart.js';
// The statistics page also includes the reading heatmap.
// Importing it here ensures it is bundled and initialized with the page.
import '../components/heatmap.js';

interface WeekData {
    read_time: number;
    pages_read: number;
    avg_pages_per_day: number;
    avg_read_time_per_day: number;
    longest_session_duration: number;
    average_session_duration: number;
}

const WEEK_SELECTOR_CLASS_STATE = {
    active: ['bg-primary-50', 'dark:bg-dark-700', 'text-primary-900', 'dark:text-white'],
    inactive: ['text-gray-600', 'dark:text-dark-200'],
} as const;

class StatisticsManager {
    private loadingIndicator: HTMLElement | null = null;
    private weekStats: HTMLElement | null = null;
    private yearlyStatsChart = new YearlyStatsChart();
    private isInitialized = false;
    private statsJsonBasePath = '/assets/json/statistics';

    // Initialize the statistics module
    init(): void {
        if (this.isInitialized) return;

        // Cache DOM elements
        this.loadingIndicator = document.getElementById('statsLoadingIndicator');
        this.weekStats = document.querySelector('.week-stats');

        // JSON base path comes from server-rendered page scope
        const base = document.body.getAttribute('data-stats-json-base');
        if (base) this.statsJsonBasePath = base;

        // Format all week date displays
        this.formatWeekDateDisplays();

        // Initialize week selector
        this.initializeWeekSelector();

        // Initialize yearly statistics chart and selector
        this.yearlyStatsChart.init(this.statsJsonBasePath);

        // Validate and reset current streak if needed
        this.validateCurrentStreak();

        this.isInitialized = true;
    }

    // Format all week date displays in the dropdown
    private formatWeekDateDisplays(): void {
        const weekOptions = document.querySelectorAll<HTMLElement>('.week-option');
        const selectedWeekText = document.getElementById('selectedWeekText');

        weekOptions.forEach((option) => {
            const startDate = option.getAttribute('data-start-date');
            const endDate = option.getAttribute('data-end-date');
            const displayEl = option.querySelector('.week-date-display');

            if (displayEl && startDate && endDate) {
                const formattedRange = DateFormatter.formatDateRange(startDate, endDate, 'long');
                displayEl.textContent = formattedRange;
            }
        });

        // Set the initially selected week text
        if (weekOptions.length > 0 && selectedWeekText) {
            const firstOption = weekOptions[0];
            const startDate = firstOption.getAttribute('data-start-date');
            const endDate = firstOption.getAttribute('data-end-date');

            if (startDate && endDate) {
                const year = startDate.substring(0, 4);
                const formattedRange = DateFormatter.formatDateRange(startDate, endDate);

                selectedWeekText.innerHTML = `<span class="font-bold">${formattedRange}</span> <span class="text-primary-400">${year}</span>`;
            }
        }
    }

    // Initialize week selector and related functionality
    private initializeWeekSelector(): void {
        const weekSelectorWrapper = document.getElementById('weekSelectorWrapper');
        const weekOptions = document.getElementById('weekOptions');
        const weekYearList = document.getElementById('weekYearList');
        const weekYearWeeksView = document.getElementById('weekYearWeeksView');
        const weekYearWeekList = document.getElementById('weekYearWeekList');
        const weekYearBackButton = document.getElementById('weekYearBackButton');
        const weekYearTitle = document.getElementById('weekYearTitle');
        const weekOptionElements = Array.from(
            document.querySelectorAll<HTMLElement>('.week-option'),
        );
        const selectedWeekText = document.getElementById('selectedWeekText');

        if (
            !weekSelectorWrapper ||
            !weekOptions ||
            !weekYearList ||
            !weekYearWeeksView ||
            !weekYearWeekList ||
            !weekYearTitle ||
            weekOptionElements.length === 0
        ) {
            return;
        }

        // Keep all week options in the weeks view container.
        weekOptionElements.forEach((option) => {
            weekYearWeekList.appendChild(option);
        });

        // Mark first option as active if none is selected.
        if (!weekOptionElements.some((option) => option.classList.contains('bg-primary-50'))) {
            weekOptionElements[0].classList.add(
                'bg-primary-50',
                'dark:bg-dark-700',
                'text-primary-900',
                'dark:text-white',
            );
            weekOptionElements[0].classList.remove('text-gray-600', 'dark:text-dark-200');
        }

        const yearOrder: string[] = [];
        weekOptionElements.forEach((option) => {
            const year = this.getWeekYear(option);
            if (year && !yearOrder.includes(year)) {
                yearOrder.push(year);
            }
        });

        let selectedYear =
            this.getWeekYear(
                weekOptionElements.find((option) => option.classList.contains('bg-primary-50')) ||
                    weekOptionElements[0],
            ) ||
            yearOrder[0] ||
            null;

        const yearOptionElements = this.buildWeekYearOptions(weekYearList, yearOrder);

        const setWeekSelectorExpanded = (expanded: boolean): void => {
            weekSelectorWrapper.setAttribute('aria-expanded', String(expanded));
        };

        const showYearList = (): void => {
            weekYearWeeksView.classList.add('hidden');
            weekYearList.classList.remove('hidden');
            this.updateActiveYearOption(yearOptionElements, selectedYear);
        };

        const showWeeksForYear = (year: string): void => {
            selectedYear = year;
            weekYearTitle.textContent = year;

            weekOptionElements.forEach((option) => {
                option.classList.toggle('hidden', this.getWeekYear(option) !== year);
            });

            weekYearList.classList.add('hidden');
            weekYearWeeksView.classList.remove('hidden');
            weekYearWeekList.scrollTop = 0;
        };

        // Handle dropdown toggle.
        weekSelectorWrapper.addEventListener('click', () => {
            if (weekOptions.classList.contains('hidden')) {
                if (selectedYear) {
                    showWeeksForYear(selectedYear);
                    this.updateActiveYearOption(yearOptionElements, selectedYear);
                } else {
                    showYearList();
                }
                weekOptions.classList.remove('hidden');
                setWeekSelectorExpanded(true);
                return;
            }

            weekOptions.classList.add('hidden');
            setWeekSelectorExpanded(false);
        });

        weekYearBackButton?.addEventListener('click', () => {
            showYearList();
        });

        yearOptionElements.forEach((yearOption) => {
            yearOption.addEventListener('click', () => {
                const year = yearOption.getAttribute('data-week-year');
                if (!year) return;
                showWeeksForYear(year);
            });
        });

        // Handle option selection.
        weekOptionElements.forEach((option) => {
            option.addEventListener('click', () => {
                const selectedIndex = option.getAttribute('data-week-index');
                const startDate = option.getAttribute('data-start-date');
                const endDate = option.getAttribute('data-end-date');

                // Update the selected week text with nice formatting
                if (startDate && endDate && selectedWeekText) {
                    const year = startDate.substring(0, 4);
                    const formattedRange = DateFormatter.formatDateRange(startDate, endDate);
                    selectedWeekText.innerHTML = `<span class="font-bold">${formattedRange}</span> <span class="text-primary-400">${year}</span>`;
                }

                selectedYear = this.getWeekYear(option);

                // Update active state in dropdown.
                this.updateActiveOption(weekOptionElements, option);
                this.updateActiveYearOption(yearOptionElements, selectedYear);

                // Load and display the selected week data.
                if (selectedIndex) {
                    void this.loadWeekData(selectedIndex);
                }

                // Close dropdown.
                weekOptions?.classList.add('hidden');
                setWeekSelectorExpanded(false);
            });
        });
    }

    private getWeekYear(option: HTMLElement): string | null {
        const dataYear = option.getAttribute('data-week-year');
        if (dataYear) return dataYear;

        const startDate = option.getAttribute('data-start-date');
        if (!startDate || startDate.length < 4) return null;
        return startDate.substring(0, 4);
    }

    private buildWeekYearOptions(container: HTMLElement, years: string[]): HTMLElement[] {
        container.replaceChildren();

        return years.map((year) => {
            const yearOption = document.createElement('button');
            yearOption.type = 'button';
            yearOption.className =
                'week-year-option w-full text-left px-4 py-2.5 cursor-pointer hover:bg-gray-100/60 dark:hover:bg-dark-700/60 text-gray-600 dark:text-dark-200 hover:text-gray-900 dark:hover:text-white transition-colors duration-200';
            yearOption.setAttribute('data-week-year', year);
            yearOption.innerHTML =
                '<div class="flex items-center justify-between">' +
                '<div class="flex items-center">' +
                '<svg class="w-4 h-4 text-primary-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">' +
                '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path>' +
                '</svg>' +
                `<span class="font-semibold">${year}</span>` +
                '</div>' +
                '<svg class="w-4 h-4 text-gray-400 dark:text-dark-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">' +
                '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>' +
                '</svg>' +
                '</div>';

            container.appendChild(yearOption);
            return yearOption;
        });
    }

    private updateActiveYearOption(allOptions: HTMLElement[], selectedYear: string | null): void {
        const selectedOption =
            allOptions.find((option) => option.getAttribute('data-week-year') === selectedYear) ||
            null;
        setActiveOption(allOptions, selectedOption, WEEK_SELECTOR_CLASS_STATE);
    }

    // Update active state for dropdown options
    private updateActiveOption(allOptions: HTMLElement[], selectedOption: HTMLElement): void {
        setActiveOption(allOptions, selectedOption, WEEK_SELECTOR_CLASS_STATE);
    }

    // Show loading indicator
    private showLoadingIndicator(): void {
        if (this.loadingIndicator) {
            this.loadingIndicator.classList.remove('hidden');
            setTimeout(() => {
                this.loadingIndicator?.classList.add('active');
            }, 10);
        }
    }

    // Hide loading indicator
    private hideLoadingIndicator(): void {
        if (this.loadingIndicator) {
            this.loadingIndicator.classList.remove('active');
            setTimeout(() => {
                this.loadingIndicator?.classList.add('hidden');
            }, 250);
        }
    }

    // Load week data and update UI
    async loadWeekData(weekIndex: string): Promise<void> {
        try {
            // Start transition out and show loading indicator
            if (this.weekStats) {
                this.weekStats.classList.add('transition-out');
                this.weekStats.classList.remove('transition-in');
            }
            this.showLoadingIndicator();

            // Wait for transition out to complete before fetching
            await new Promise((resolve) => setTimeout(resolve, 200));

            const response = await fetch(`${this.statsJsonBasePath}/week_${weekIndex}.json`);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const weekData = (await response.json()) as WeekData;

            // Update UI with the loaded data
            this.updateWeekStats(weekData);

            // Hide loading indicator after data is loaded
            this.hideLoadingIndicator();
        } catch (error) {
            console.error('Error loading week data:', error);

            // Hide loading indicator even on error
            this.hideLoadingIndicator();

            // Try to transition back in
            if (this.weekStats) {
                this.weekStats.classList.remove('transition-out');
                this.weekStats.classList.add('transition-in');
            }
        }
    }

    // Update the UI with week data
    private updateWeekStats(weekData: WeekData): void {
        const weekReadTime = document.getElementById('weekReadTime');
        const weekPagesRead = document.getElementById('weekPagesRead');
        const weekAvgPagesPerDay = document.getElementById('weekAvgPagesPerDay');
        const weekAvgReadTimePerDay = document.getElementById('weekAvgReadTimePerDay');
        const weekLongestSession = document.getElementById('weekLongestSession');
        const weekAverageSession = document.getElementById('weekAverageSession');

        // Update the values
        if (weekReadTime)
            weekReadTime.textContent = DataFormatter.formatReadTime(weekData.read_time);
        if (weekPagesRead) weekPagesRead.textContent = String(weekData.pages_read);
        if (weekAvgPagesPerDay)
            weekAvgPagesPerDay.textContent = DataFormatter.formatAvgPages(
                weekData.avg_pages_per_day,
            );
        if (weekAvgReadTimePerDay)
            weekAvgReadTimePerDay.textContent = `${Math.floor(weekData.avg_read_time_per_day / 60)}m`;
        if (weekLongestSession)
            weekLongestSession.textContent = DataFormatter.formatReadTime(
                weekData.longest_session_duration,
            );
        if (weekAverageSession)
            weekAverageSession.textContent = DataFormatter.formatReadTime(
                weekData.average_session_duration,
            );

        // Use requestAnimationFrame to ensure DOM updates before animation
        requestAnimationFrame(() => {
            // Slight delay to ensure values are updated before transitioning
            setTimeout(() => {
                if (this.weekStats) {
                    this.weekStats.classList.remove('transition-out');
                    this.weekStats.classList.add('transition-in');
                }
            }, 50);
        });
    }

    // Validate current streak and reset to 0 if the last streak date is not today
    private validateCurrentStreak(): void {
        const streakElement = document.getElementById('currentStreakDays');
        const dateRangeElement = document.getElementById('currentStreakDateRange');
        const daysTextElement = document.getElementById('currentStreakDaysText');

        if (!streakElement) return;

        const lastStreakDate = streakElement.getAttribute('data-last-streak-date');
        if (!lastStreakDate) return;

        // Get today's date in YYYY-MM-DD format
        const today = new Date();
        const todayStr =
            today.getFullYear() +
            '-' +
            String(today.getMonth() + 1).padStart(2, '0') +
            '-' +
            String(today.getDate()).padStart(2, '0');

        // If the last streak date is not today, reset the streak to 0
        if (lastStreakDate !== todayStr) {
            streakElement.textContent = '0';

            // Update the day/days text
            if (daysTextElement) {
                daysTextElement.textContent = translation.get('days_label', 0);
            }

            // Clear the date range
            if (dateRangeElement) {
                dateRangeElement.textContent = '';
            }
        }
    }
}

// Create and export the statistics manager instance
const statisticsManager = new StatisticsManager();

// Initialize when DOM is ready
async function initStats(): Promise<void> {
    await translation.init();
    new SectionToggle();
    statisticsManager.init();
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => initStats());
} else {
    initStats();
}

// Export for module use
export { statisticsManager as default, StatisticsManager, DateFormatter, DataFormatter };
