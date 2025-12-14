/**
 * KoReader Statistics Viewer Module
 * Handles loading and displaying reading statistics
 */

import { translation } from '../shared/i18n.js';
import { SectionToggle } from '../components/section-toggle.js';
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

class StatisticsManager {
    private loadingIndicator: HTMLElement | null = null;
    private weekStats: HTMLElement | null = null;
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
                const formattedRange = DateFormatter.formatDateRange(startDate, endDate);
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
        const weekOptionElements = document.querySelectorAll<HTMLElement>('.week-option');
        const selectedWeekText = document.getElementById('selectedWeekText');

        // Handle dropdown toggle
        if (weekSelectorWrapper && weekOptions) {
            weekSelectorWrapper.addEventListener('click', () => {
                weekOptions.classList.toggle('hidden');
            });
        }

        // Handle option selection
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

                // Update active state in dropdown
                this.updateActiveOption(weekOptionElements, option);

                // Load and display the selected week data
                if (selectedIndex) {
                    this.loadWeekData(selectedIndex);
                }

                // Close dropdown
                weekOptions?.classList.add('hidden');
            });
        });

        // Mark first option as active if none is selected
        if (
            weekOptionElements.length > 0 &&
            !weekOptionElements[0].classList.contains('bg-primary-50')
        ) {
            weekOptionElements[0].classList.add(
                'bg-primary-50',
                'dark:bg-dark-700',
                'text-primary-900',
                'dark:text-white',
            );
            weekOptionElements[0].classList.remove('text-gray-600', 'dark:text-dark-200');
        }
    }

    // Update active state for dropdown options
    private updateActiveOption(
        allOptions: NodeListOf<HTMLElement>,
        selectedOption: HTMLElement,
    ): void {
        allOptions.forEach((el) => {
            // Remove both light and dark mode active classes
            el.classList.remove(
                'bg-primary-50',
                'dark:bg-dark-700',
                'text-primary-900',
                'dark:text-white',
                'bg-green-50',
                'text-green-900',
            );
            // Reset to default text color
            el.classList.add('text-gray-600', 'dark:text-dark-200');
        });

        // Add appropriate active classes based on the context (week or year selector)
        if (selectedOption.closest('#weekOptions')) {
            selectedOption.classList.add(
                'bg-primary-50',
                'dark:bg-dark-700',
                'text-primary-900',
                'dark:text-white',
            );
        } else {
            selectedOption.classList.add(
                'bg-green-50',
                'dark:bg-dark-700',
                'text-green-900',
                'dark:text-white',
            );
        }
        selectedOption.classList.remove('text-gray-600', 'dark:text-dark-200');
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
                daysTextElement.textContent = 'days';
            }

            // Clear the date range
            if (dateRangeElement) {
                dateRangeElement.textContent = '';
            }
        }
    }
}

// Date formatting utilities
class DateFormatter {
    // Parse ISO date string and return a Date object
    static parseISODate(dateStr: string): Date {
        try {
            return new Date(dateStr);
        } catch {
            console.error('Error parsing date:', dateStr);
            return new Date(); // Return current date as fallback
        }
    }

    // Format date as "D Month" (e.g. "17 March")
    static formatDateNice(dateObj: Date): string {
        const monthKeys = [
            'january',
            'february',
            'march',
            'april',
            'may',
            'june',
            'july',
            'august',
            'september',
            'october',
            'november',
            'december',
        ];
        return `${dateObj.getDate()} ${translation.get(monthKeys[dateObj.getMonth()])}`;
    }

    // Format a date range nicely (e.g. "17-23 March" or "28 Feb - 5 March")
    static formatDateRange(startDateStr: string, endDateStr: string): string {
        const startDate = this.parseISODate(startDateStr);
        const endDate = this.parseISODate(endDateStr);

        const startDay = startDate.getDate();
        const startMonth = startDate.getMonth();
        const endDay = endDate.getDate();
        const endMonth = endDate.getMonth();
        const startYear = startDate.getFullYear();
        const endYear = endDate.getFullYear();

        const monthKeys = [
            'january.short',
            'february.short',
            'march.short',
            'april.short',
            'may.short',
            'june.short',
            'july.short',
            'august.short',
            'september.short',
            'october.short',
            'november.short',
            'december.short',
        ];
        const months = monthKeys.map((k) => translation.get(k));

        // If same month
        if (startMonth === endMonth && startYear === endYear) {
            return `${startDay}-${endDay} ${months[startMonth]}`;
        }
        // Different months
        else {
            return `${startDay} ${months[startMonth]} - ${endDay} ${months[endMonth]}`;
        }
    }
}

// Data formatting utilities
class DataFormatter {
    // Format read time from seconds to hours and minutes
    static formatReadTime(seconds: number | null | undefined): string {
        if (seconds === null || seconds === undefined) {
            return '--';
        }
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        } else {
            return `${minutes}m`;
        }
    }

    // Format average pages with one decimal place
    static formatAvgPages(avg: number): string {
        return (Math.floor(avg * 10) / 10).toFixed(1);
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
