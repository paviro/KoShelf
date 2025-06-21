/**
 * KoReader Statistics Viewer
 * Handles loading and displaying reading statistics
 */

// Initialize when the DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    // Format all week date displays
    formatWeekDateDisplays();
    
    initializeWeekSelector();
    
    // Add transition CSS directly to ensure smooth animations
    const style = document.createElement('style');
    style.textContent = `
        .week-stats {
            transition: opacity 0.3s ease-out, transform 0.3s ease-out;
        }
        .week-stats.transition-out {
            opacity: 0.3;
        }
        .week-stats.transition-in {
            opacity: 1;
            transform: translateY(0);
        }
        #statsLoadingIndicator {
            transition: opacity 0.25s ease-in-out;
            opacity: 0;
        }
        #statsLoadingIndicator.active {
            opacity: 1;
        }
    `;
    document.head.appendChild(style);
    
    // Ensure initial data is fully visible - don't gray it out
    const weekStats = document.querySelector('.week-stats');
    if (weekStats) {
        weekStats.classList.remove('transition-out');
        weekStats.classList.add('transition-in');
    }
});

// Parse ISO date string and return a Date object
function parseISODate(dateStr) {
    // Try to parse the date string
    try {
        // If it's a standard ISO format (YYYY-MM-DD)
        return new Date(dateStr);
    } catch (e) {
        console.error('Error parsing date:', dateStr);
        return new Date(); // Return current date as fallback
    }
}

// Format date as "D Month" (e.g. "17 March")
function formatDateNice(dateObj) {
    const months = [
        'January', 'February', 'March', 'April', 'May', 'June',
        'July', 'August', 'September', 'October', 'November', 'December'
    ];
    
    return `${dateObj.getDate()} ${months[dateObj.getMonth()]}`;
}

// Format a date range nicely (e.g. "17-23 March" or "28 Feb - 5 March")
function formatDateRange(startDateStr, endDateStr) {
    const startDate = parseISODate(startDateStr);
    const endDate = parseISODate(endDateStr);
    
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

// Format all week date displays in the dropdown
function formatWeekDateDisplays() {
    const weekOptions = document.querySelectorAll('.week-option');
    const selectedWeekText = document.getElementById('selectedWeekText');
    
    weekOptions.forEach(option => {
        const startDate = option.getAttribute('data-start-date');
        const endDate = option.getAttribute('data-end-date');
        const displayEl = option.querySelector('.week-date-display');
        
        if (displayEl && startDate && endDate) {
            const formattedRange = formatDateRange(startDate, endDate);
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
            const formattedRange = formatDateRange(startDate, endDate);
            
            selectedWeekText.innerHTML = `<span class="font-bold">${formattedRange}</span> <span class="text-primary-400">${year}</span>`;
        }
    }
}

// Initialize week selector and related functionality
function initializeWeekSelector() {
    const weekSelectorWrapper = document.getElementById('weekSelectorWrapper');
    const weekOptions = document.getElementById('weekOptions');
    const dropdownArrow = document.getElementById('dropdownArrow');
    const weekOptionElements = document.querySelectorAll('.week-option');
    const selectedWeekText = document.getElementById('selectedWeekText');
    const loadingIndicator = document.getElementById('statsLoadingIndicator');
    
    // Handle dropdown toggle
    if (weekSelectorWrapper) {
        weekSelectorWrapper.addEventListener('click', function() {
            weekOptions.classList.toggle('hidden');
            dropdownArrow.classList.toggle('rotate-180');
        });
    }
    
    // Handle option selection
    weekOptionElements.forEach(option => {
        option.addEventListener('click', function() {
            const selectedIndex = this.getAttribute('data-week-index');
            const startDate = this.getAttribute('data-start-date');
            const endDate = this.getAttribute('data-end-date');
            
            // Update the selected week text with nice formatting
            if (startDate && endDate) {
                const year = startDate.substring(0, 4);
                const formattedRange = formatDateRange(startDate, endDate);
                selectedWeekText.innerHTML = `<span class="font-bold">${formattedRange}</span> <span class="text-primary-400">${year}</span>`;
            }
            
            // Update active state in dropdown
            weekOptionElements.forEach(el => {
                el.classList.remove('bg-dark-750', 'text-white');
                el.classList.add('text-dark-200');
            });
            this.classList.add('bg-dark-750', 'text-white');
            this.classList.remove('text-dark-200');
            
            // Load and display the selected week data
            loadWeekData(selectedIndex);
            
            // Close dropdown
            weekOptions.classList.add('hidden');
            dropdownArrow.classList.remove('rotate-180');
        });
    });
    
    // Close dropdown when clicking outside
    document.addEventListener('click', function(event) {
        if (!weekSelectorWrapper?.contains(event.target) && !weekOptions?.contains(event.target)) {
            weekOptions?.classList.add('hidden');
            dropdownArrow?.classList.remove('rotate-180');
        }
    });
    
    // If we have week options and the first one isn't already marked as active, load the first week data
    if (weekOptionElements.length > 0 && !weekOptionElements[0].classList.contains('bg-dark-750')) {
        const firstWeekIndex = weekOptionElements[0].getAttribute('data-week-index');
        
        // Mark the first option as active
        weekOptionElements[0].classList.add('bg-dark-750', 'text-white');
        weekOptionElements[0].classList.remove('text-dark-200');
        
        // We don't need to load data on initial page load since it's already in the HTML
        // But we should initialize any other UI elements or behaviors
    }
}

// Format read time from seconds to hours and minutes
function formatReadTime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
}

// Format average pages with one decimal place
function formatAvgPages(avg) {
    return (Math.floor(avg * 10) / 10).toFixed(1);
}

// Show loading indicator
function showLoadingIndicator() {
    const loadingIndicator = document.getElementById('statsLoadingIndicator');
    if (loadingIndicator) {
        loadingIndicator.classList.remove('hidden');
        setTimeout(() => {
            loadingIndicator.classList.add('active');
        }, 10);
    }
}

// Hide loading indicator
function hideLoadingIndicator() {
    const loadingIndicator = document.getElementById('statsLoadingIndicator');
    if (loadingIndicator) {
        loadingIndicator.classList.remove('active');
        setTimeout(() => {
            loadingIndicator.classList.add('hidden');
        }, 250);
    }
}

// Load week data and update UI
async function loadWeekData(weekIndex) {
    try {
        // Start transition out and show loading indicator
        const weekStats = document.querySelector('.week-stats');
        weekStats.classList.add('transition-out');
        weekStats.classList.remove('transition-in');
        showLoadingIndicator();
        
        // Wait for transition out to complete before fetching
        await new Promise(resolve => setTimeout(resolve, 200));
        
        const response = await fetch(`/assets/json/week_${weekIndex}.json`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const weekData = await response.json();
        
        // Update UI with the loaded data
        updateWeekStats(weekData);
        
        // Hide loading indicator after data is loaded
        hideLoadingIndicator();
        
    } catch (error) {
        console.error('Error loading week data:', error);
        
        // Hide loading indicator even on error
        hideLoadingIndicator();
        
        // Try to transition back in
        const weekStats = document.querySelector('.week-stats');
        if (weekStats) {
            weekStats.classList.remove('transition-out');
            weekStats.classList.add('transition-in');
        }
    }
}

// Update the UI with week data
function updateWeekStats(weekData) {
    const weekReadTime = document.getElementById('weekReadTime');
    const weekPagesRead = document.getElementById('weekPagesRead');
    const weekAvgPagesPerDay = document.getElementById('weekAvgPagesPerDay');
    const weekAvgReadTimePerDay = document.getElementById('weekAvgReadTimePerDay');
    const weekStats = document.querySelector('.week-stats');
    
    // Update the values
    weekReadTime.textContent = formatReadTime(weekData.read_time);
    weekPagesRead.textContent = weekData.pages_read;
    weekAvgPagesPerDay.textContent = formatAvgPages(weekData.avg_pages_per_day);
    weekAvgReadTimePerDay.textContent = `${Math.floor(weekData.avg_read_time_per_day / 60)}m`;
    
    // Use requestAnimationFrame to ensure DOM updates before animation
    requestAnimationFrame(() => {
        // Slight delay to ensure values are updated before transitioning
        setTimeout(() => {
            weekStats.classList.remove('transition-out');
            weekStats.classList.add('transition-in');
        }, 50);
    });
} 