/**
 * KoReader Statistics Viewer Module
 * Handles loading and displaying reading statistics
 */

class StatisticsManager {
    constructor() {
        this.loadingIndicator = null;
        this.weekStats = null;
        this.isInitialized = false;
    }

    // Initialize the statistics module
    init() {
        if (this.isInitialized) return;
        
        // Cache DOM elements
        this.loadingIndicator = document.getElementById('statsLoadingIndicator');
        this.weekStats = document.querySelector('.week-stats');
        
        // Format all week date displays
        this.formatWeekDateDisplays();
        
        // Initialize week selector
        this.initializeWeekSelector();
        
        this.isInitialized = true;
    }

    // Format all week date displays in the dropdown
    formatWeekDateDisplays() {
        const weekOptions = document.querySelectorAll('.week-option');
        const selectedWeekText = document.getElementById('selectedWeekText');
        
        weekOptions.forEach(option => {
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
    initializeWeekSelector() {
        const weekSelectorWrapper = document.getElementById('weekSelectorWrapper');
        const weekOptions = document.getElementById('weekOptions');
        const dropdownArrow = document.getElementById('dropdownArrow');
        const weekOptionElements = document.querySelectorAll('.week-option');
        const selectedWeekText = document.getElementById('selectedWeekText');
        
        // Handle dropdown toggle
        if (weekSelectorWrapper) {
            weekSelectorWrapper.addEventListener('click', () => {
                weekOptions.classList.toggle('hidden');
                dropdownArrow.classList.toggle('rotate-180');
            });
        }
        
        // Handle option selection
        weekOptionElements.forEach(option => {
            option.addEventListener('click', (e) => {
                const selectedIndex = option.getAttribute('data-week-index');
                const startDate = option.getAttribute('data-start-date');
                const endDate = option.getAttribute('data-end-date');
                
                // Update the selected week text with nice formatting
                if (startDate && endDate) {
                    const year = startDate.substring(0, 4);
                    const formattedRange = DateFormatter.formatDateRange(startDate, endDate);
                    selectedWeekText.innerHTML = `<span class="font-bold">${formattedRange}</span> <span class="text-primary-400">${year}</span>`;
                }
                
                // Update active state in dropdown
                this.updateActiveOption(weekOptionElements, option);
                
                // Load and display the selected week data
                this.loadWeekData(selectedIndex);
                
                // Close dropdown
                weekOptions.classList.add('hidden');
                dropdownArrow.classList.remove('rotate-180');
            });
        });
        
        // Close dropdown when clicking outside
        document.addEventListener('click', (event) => {
            if (!weekSelectorWrapper?.contains(event.target) && !weekOptions?.contains(event.target)) {
                weekOptions?.classList.add('hidden');
                dropdownArrow?.classList.remove('rotate-180');
            }
        });
        
        // Mark first option as active if none is selected
        if (weekOptionElements.length > 0 && !weekOptionElements[0].classList.contains('bg-primary-50')) {
            weekOptionElements[0].classList.add('bg-primary-50', 'dark:bg-dark-700', 'text-primary-900', 'dark:text-white');
            weekOptionElements[0].classList.remove('text-gray-600', 'dark:text-dark-200');
        }
    }

    // Update active state for dropdown options
    updateActiveOption(allOptions, selectedOption) {
        allOptions.forEach(el => {
            // Remove both light and dark mode active classes
            el.classList.remove('bg-primary-50', 'dark:bg-dark-700', 'text-primary-900', 'dark:text-white', 'bg-green-50', 'text-green-900');
            // Reset to default text color
            el.classList.add('text-gray-600', 'dark:text-dark-200');
        });

        // Add appropriate active classes based on the context (week or year selector)
        if (selectedOption.closest('#weekOptions')) {
            selectedOption.classList.add('bg-primary-50', 'dark:bg-dark-700', 'text-primary-900', 'dark:text-white');
        } else {
            selectedOption.classList.add('bg-green-50', 'dark:bg-dark-700', 'text-green-900', 'dark:text-white');
        }
        selectedOption.classList.remove('text-gray-600', 'dark:text-dark-200');
    }

    // Show loading indicator
    showLoadingIndicator() {
        if (this.loadingIndicator) {
            this.loadingIndicator.classList.remove('hidden');
            setTimeout(() => {
                this.loadingIndicator.classList.add('active');
            }, 10);
        }
    }

    // Hide loading indicator
    hideLoadingIndicator() {
        if (this.loadingIndicator) {
            this.loadingIndicator.classList.remove('active');
            setTimeout(() => {
                this.loadingIndicator.classList.add('hidden');
            }, 250);
        }
    }

    // Load week data and update UI
    async loadWeekData(weekIndex) {
        try {
            // Start transition out and show loading indicator
            if (this.weekStats) {
                this.weekStats.classList.add('transition-out');
                this.weekStats.classList.remove('transition-in');
            }
            this.showLoadingIndicator();
            
            // Wait for transition out to complete before fetching
            await new Promise(resolve => setTimeout(resolve, 200));
            
            const response = await fetch(`/assets/json/statistics/week_${weekIndex}.json`);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const weekData = await response.json();
            
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
    updateWeekStats(weekData) {
        const weekReadTime = document.getElementById('weekReadTime');
        const weekPagesRead = document.getElementById('weekPagesRead');
        const weekAvgPagesPerDay = document.getElementById('weekAvgPagesPerDay');
        const weekAvgReadTimePerDay = document.getElementById('weekAvgReadTimePerDay');
        const weekLongestSession = document.getElementById('weekLongestSession');
        const weekAverageSession = document.getElementById('weekAverageSession');
        
        // Update the values
        if (weekReadTime) weekReadTime.textContent = DataFormatter.formatReadTime(weekData.read_time);
        if (weekPagesRead) weekPagesRead.textContent = weekData.pages_read;
        if (weekAvgPagesPerDay) weekAvgPagesPerDay.textContent = DataFormatter.formatAvgPages(weekData.avg_pages_per_day);
        if (weekAvgReadTimePerDay) weekAvgReadTimePerDay.textContent = `${Math.floor(weekData.avg_read_time_per_day / 60)}m`;
        if (weekLongestSession) weekLongestSession.textContent = DataFormatter.formatReadTime(weekData.longest_session_duration);
        if (weekAverageSession) weekAverageSession.textContent = DataFormatter.formatReadTime(weekData.average_session_duration);
        
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
}

// Date formatting utilities
class DateFormatter {
    // Parse ISO date string and return a Date object
    static parseISODate(dateStr) {
        try {
            return new Date(dateStr);
        } catch (e) {
            console.error('Error parsing date:', dateStr);
            return new Date(); // Return current date as fallback
        }
    }

    // Format date as "D Month" (e.g. "17 March")
    static formatDateNice(dateObj) {
        const months = [
            'January', 'February', 'March', 'April', 'May', 'June',
            'July', 'August', 'September', 'October', 'November', 'December'
        ];
        
        return `${dateObj.getDate()} ${months[dateObj.getMonth()]}`;
    }

    // Format a date range nicely (e.g. "17-23 March" or "28 Feb - 5 March")
    static formatDateRange(startDateStr, endDateStr) {
        const startDate = this.parseISODate(startDateStr);
        const endDate = this.parseISODate(endDateStr);
        
        const startDay = startDate.getDate();
        const startMonth = startDate.getMonth();
        const endDay = endDate.getDate();
        const endMonth = endDate.getMonth();
        const startYear = startDate.getFullYear();
        const endYear = endDate.getFullYear();
        
        const months = [
            'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
            'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
        ];
        
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
    static formatReadTime(seconds) {
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
    static formatAvgPages(avg) {
        return (Math.floor(avg * 10) / 10).toFixed(1);
    }
}

// Create and export the statistics manager instance
const statisticsManager = new StatisticsManager();

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => statisticsManager.init());
} else {
    statisticsManager.init();
}

// Export for module use
export { statisticsManager as default, StatisticsManager, DateFormatter, DataFormatter }; 