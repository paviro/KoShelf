/**
 * KoReader Activity Heatmap Module
 * Handles loading and displaying reading activity heatmap with year selection
 */

class ActivityHeatmap {
    constructor() {
        this.activityData = null;
        this.availableYears = [];
        this.currentYear = null;
        this.isInitialized = false;
        this.resizeObserver = null;
    }

    // Initialize the heatmap module
    async init() {
        if (this.isInitialized) return;
        
        try {
            // Get available years from the template (no need to load from JSON)
            this.getAvailableYearsFromTemplate();
            
            // Initialize year selector
            this.initializeYearSelector();
            
            // Load data for the current year (most recent by default)
            if (this.availableYears.length > 0) {
                this.currentYear = this.availableYears[0]; // Most recent year first
                await this.loadYearData(this.currentYear);
                this.initializeHeatmap();
            }
            
            this.isInitialized = true;
            
        } catch (error) {
            console.error('Error initializing heatmap:', error);
        }
    }

    // Get available years from the template-rendered year options
    getAvailableYearsFromTemplate() {
        const yearOptions = document.querySelectorAll('.year-option');
        this.availableYears = Array.from(yearOptions).map(option => 
            parseInt(option.getAttribute('data-year'))
        );
        
        // Fallback to current year if no options are available
        if (this.availableYears.length === 0) {
            this.availableYears = [new Date().getFullYear()];
        }
    }

    // Load activity data for a specific year
    async loadYearData(year) {
        try {
            const response = await fetch(`/assets/json/statistics/daily_activity_${year}.json`);
            if (!response.ok) {
                throw new Error(`Failed to load activity data for ${year}`);
            }
            
            this.activityData = await response.json();
            this.currentYear = year;
            
        } catch (error) {
            console.error(`Error loading activity data for ${year}:`, error);
            this.activityData = [];
        }
    }

    // Initialize year selector functionality
    initializeYearSelector() {
        const yearSelectorWrapper = document.getElementById('yearSelectorWrapper');
        const yearOptions = document.getElementById('yearOptions');
        const yearDropdownArrow = document.getElementById('yearDropdownArrow');
        const selectedYearText = document.getElementById('selectedYearText');
        
        if (!yearSelectorWrapper || !yearOptions) return;
        
        // Add click handlers to existing year options
        this.setupYearOptionHandlers();
        
        // Handle dropdown toggle
        yearSelectorWrapper.addEventListener('click', () => {
            yearOptions.classList.toggle('hidden');
            yearDropdownArrow.classList.toggle('rotate-180');
        });
        
        // Close dropdown when clicking outside
        document.addEventListener('click', (e) => {
            if (!yearSelectorWrapper.contains(e.target)) {
                yearOptions.classList.add('hidden');
                yearDropdownArrow.classList.remove('rotate-180');
            }
        });
    }

    // Setup click handlers for existing year options
    setupYearOptionHandlers() {
        const yearOptionElements = document.querySelectorAll('.year-option');
        
        yearOptionElements.forEach((option) => {
            // Add click handler
            option.addEventListener('click', async (e) => {
                const selectedYear = parseInt(option.getAttribute('data-year'));
                await this.selectYear(selectedYear);
                
                // Update UI
                this.updateActiveYearOption(option);
                this.updateSelectedYearText(selectedYear);
                
                // Close dropdown
                document.getElementById('yearOptions').classList.add('hidden');
                document.getElementById('yearDropdownArrow').classList.remove('rotate-180');
            });
        });
    }

    // Select a specific year and reload heatmap
    async selectYear(year) {
        if (year === this.currentYear) return;
        
        try {
            // Show loading state
            this.showLoadingState();
            
            // Load new year data
            await this.loadYearData(year);
            
            // Reinitialize heatmap with new data
            this.initializeHeatmap();
            
        } catch (error) {
            console.error(`Error selecting year ${year}:`, error);
        } finally {
            this.hideLoadingState();
        }
    }

    // Update active year option in dropdown
    updateActiveYearOption(selectedOption) {
        const allOptions = document.querySelectorAll('.year-option');
        allOptions.forEach(opt => {
            opt.classList.remove('bg-dark-700', 'text-white');
            opt.classList.add('text-dark-200');
        });
        
        selectedOption.classList.add('bg-dark-700', 'text-white');
        selectedOption.classList.remove('text-dark-200');
    }

    // Update selected year text
    updateSelectedYearText(year) {
        const selectedYearText = document.getElementById('selectedYearText');
        if (selectedYearText) {
            selectedYearText.innerHTML = `<span class="font-bold">${year}</span>`;
        }
    }

    // Show loading state
    showLoadingState() {
        const heatmapGrid = document.getElementById('heatmapGrid');
        if (heatmapGrid) {
            heatmapGrid.style.opacity = '0.5';
        }
    }

    // Hide loading state
    hideLoadingState() {
        const heatmapGrid = document.getElementById('heatmapGrid');
        if (heatmapGrid) {
            heatmapGrid.style.opacity = '1';
        }
    }

    // Initialize and render the heatmap
    initializeHeatmap() {
        if (!this.activityData) return;

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
    setupHeightSync() {
        const heatmapGrid = document.getElementById('heatmapGrid');
        const dayLabels = document.getElementById('dayLabels');
        
        if (heatmapGrid && dayLabels) {
            dayLabels.style.height = heatmapGrid.offsetHeight + 'px';
        }
    }

    // Process activity data and find maximum activity level
    processActivityData(activityData) {
        const activityMap = new Map();
        let maxActivity = 0;
        
        // Find max reading time and fill map
        activityData.forEach(day => {
            if (day.read_time > maxActivity) {
                maxActivity = day.read_time;
            }
            activityMap.set(day.date, { 
                pages: day.pages_read, 
                read: day.read_time 
            });
        });
        
        return { activityMap, maxActivity };
    }

    // Fill heatmap cells with activity data
    fillHeatmapCells(activityMap, maxActivity) {
        const cells = document.querySelectorAll('.activity-cell');
        
        cells.forEach(cell => {
            // Calculate the date for this cell
            const cellDate = this.calculateCellDate(cell, this.currentYear);
            const dateStr = DateUtils.formatDateAsISO(cellDate);
            
            // Get activity level for this date (use reading time)
            const activityObj = activityMap.get(dateStr) || { pages: 0, read: 0 };
            const activity = activityObj.read;
            
            // Normalize and apply activity level
            const activityLevel = this.normalizeActivityLevel(activity, maxActivity);
            this.applyCellStyling(cell, activityLevel, dateStr, activityObj);
        });
    }

    // Calculate the date represented by a heatmap cell for a specific year
    calculateCellDate(cell, year) {
        const weekIndex = parseInt(cell.getAttribute('data-week'));
        const dayIndex = parseInt(cell.getAttribute('data-day'));
        
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
    normalizeActivityLevel(activity, maxActivity) {
        let activityLevel = 0;
        if (activity > 0) {
            if (maxActivity <= 4) {
                activityLevel = activity;
            } else {
                activityLevel = Math.ceil((activity / maxActivity) * 4);
            }
        }
        return activityLevel;
    }

    // Apply styling and interactions to a heatmap cell
    applyCellStyling(cell, activityLevel, dateStr, activityObj) {
        const colorClasses = [
            ['bg-gray-200', 'dark:bg-dark-700'],       // 0 (no activity)
            ['bg-green-100', 'dark:bg-green-900'],     // 1 (low)
            ['bg-green-300', 'dark:bg-green-700'],     // 2 (medium)
            ['bg-green-500', 'dark:bg-green-500'],     // 3 (high)
            ['bg-green-600', 'dark:bg-green-300']      // 4 (very high)
        ];
        
        // Remove all possible color classes
        colorClasses.flat().forEach(cls => cell.classList.remove(cls));
        
        // Add both light and dark mode classes for the current activity level
        colorClasses[activityLevel].forEach(cls => cell.classList.add(cls));
        
        // Set tooltip (reading time focused)
        const readLabel = DateUtils.formatDuration(activityObj.read);
        cell.setAttribute('title', `${dateStr}: ${readLabel}, ${activityObj.pages} pages`);
        
        // Add hover functionality
        this.addCellHoverEffects(cell);
    }

    // Add hover effects to a cell
    addCellHoverEffects(cell) {
        cell.addEventListener('mouseover', function() {
            this.classList.add('ring-1', 'ring-white', 'z-10');
        });
        
        cell.addEventListener('mouseout', function() {
            this.classList.remove('ring-1', 'ring-white', 'z-10');
        });
    }

    // Auto-scroll to show current month positioned towards the right
    scrollToCurrentMonth() {
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
            const targetPosition = (currentWeek * weekWidth) - (containerWidth * 0.7);
            
            // Ensure we don't scroll past the beginning or end
            const maxScroll = heatmapWidth - containerWidth;
            const scrollPosition = Math.max(0, Math.min(targetPosition, maxScroll));
            
            scrollContainer.scrollLeft = scrollPosition;
        }
    }

    // Scroll to end of year for past years
    scrollToEndOfYear() {
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
    calculateCurrentWeek(today) {
        const baseYear = today.getFullYear();
        const janFirst = new Date(baseYear, 0, 1);
        
        // Find the Monday on (or before) Jan 1
        const janDayOfWeek = janFirst.getDay();
        const shiftToMonday = janDayOfWeek === 0 ? -6 : 1 - janDayOfWeek;
        const firstMonday = new Date(janFirst);
        firstMonday.setDate(janFirst.getDate() + shiftToMonday);
        
        // Calculate which week the current date falls into
        const daysDiff = Math.floor((today - firstMonday) / (1000 * 60 * 60 * 24));
        return Math.floor(daysDiff / 7);
    }

    // Setup resize observer to keep heights in sync
    setupResizeObserver() {
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
    destroy() {
        if (this.resizeObserver) {
            this.resizeObserver.disconnect();
            this.resizeObserver = null;
        }
        this.isInitialized = false;
    }
}

// Date utility functions
class DateUtils {
    // Format date as YYYY-MM-DD for lookup
    static formatDateAsISO(date) {
        return `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`;
    }

    // Format seconds into "Xh Ym" style
    static formatDuration(secs) {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const parts = [];
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