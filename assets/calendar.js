// Calendar module: provides initializeCalendar() to bootstrap the reading calendar UI
// All logic is self-contained – nothing is written to or read from the global `window` object.
// Consumers can use:
//   import { initializeCalendar } from '/assets/js/calendar.js';
//   initializeCalendar();

let calendar;
let currentEvents = [];
let currentBooks = {};
let monthlyDataCache = new Map(); // Cache for monthly data: month -> {events, books}
let availableMonths = []; // List of months that have data
let currentDisplayedMonth = null;

// Exported entry point
export function initializeCalendar() {
    // First, load the list of available months
    loadAvailableMonths().then(() => {
        // Load calendar data for current month and its neighbours
        const now = new Date();
        const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
        const [prevMonth, nextMonth] = getAdjacentMonths(currentMonth);

        (async () => {
            // Load current, previous and next month (if they exist)
            await Promise.all([
                fetchMonthData(prevMonth),
                fetchMonthData(currentMonth),
                fetchMonthData(nextMonth)
            ]);

            currentDisplayedMonth = currentMonth;
            refreshAggregatedData();

            initializeEventCalendar(currentEvents);

            // Populate statistics widgets for the initial month
            updateMonthlyStats(now);

            // Wire up DOM interaction handlers (today / prev / next / modal)
            setupEventHandlers();
        })();
    });
}

// Load the list of available months
async function loadAvailableMonths() {
    try {
        const response = await fetch('/assets/json/calendar/available_months.json');
        if (response.ok) {
            availableMonths = await response.json();
        } else {
            console.warn('Could not load available months list');
            availableMonths = [];
        }
    } catch (error) {
        console.warn('Error loading available months:', error);
        availableMonths = [];
    }
}

// Load and update calendar for a specific month
async function updateDisplayedMonth(targetMonth) {
    // Skip if this month is already displayed
    if (currentDisplayedMonth === targetMonth) {
        return;
    }

    try {
        const [prevMonth, nextMonth] = getAdjacentMonths(targetMonth);

        // Load target, previous and next months in parallel (will use cache when possible)
        await Promise.all([
            fetchMonthData(prevMonth),
            fetchMonthData(targetMonth),
            fetchMonthData(nextMonth)
        ]);

        currentDisplayedMonth = targetMonth;

        // Re-aggregate events/books from everything we have cached
        refreshAggregatedData();

        // Update calendar events with all cached data
        if (calendar) {
            const mapEvents = evts => evts.map(ev => {
                const book = currentBooks[ev.book_id] || {};
                return {
                    id: ev.book_id,
                    title: book.title || 'Unknown Book',
                    start: ev.start,
                    end: ev.end || ev.start,
                    allDay: true,
                    backgroundColor: book.color || getEventColor(ev),
                    borderColor: book.color || getEventColor(ev),
                    textColor: '#ffffff',
                    extendedProps: {
                        ...ev,
                        book_title: book.title || 'Unknown Book',
                        authors: book.authors || [],
                        book_path: book.book_path,
                        book_cover: book.book_cover,
                        color: book.color,
                        md5: ev.book_id
                    }
                };
            });

            calendar.setOption('events', mapEvents(currentEvents));

            // Update monthly statistics now that we have fresh data
            const [yr, mo] = targetMonth.split('-');
            const statsDate = new Date(Number(yr), Number(mo) - 1, 1);
            updateMonthlyStats(statsDate);
        }
    } catch (error) {
        console.error(`Failed to load calendar data for ${targetMonth}:`, error);
    }
}

// Load calendar data from monthly JSON files
async function fetchMonthData(targetMonth = null) {
    try {
        // If no target month specified, use current month
        if (!targetMonth) {
            const now = new Date();
            targetMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
        }
        
        // Check if month data is already cached
        if (monthlyDataCache.has(targetMonth)) {
            return monthlyDataCache.get(targetMonth);
        }
        
        // Check if this month has data available
        if (availableMonths.length > 0 && !availableMonths.includes(targetMonth)) {
            console.info(`No calendar data available for ${targetMonth}`);
            return { events: [], books: {} }; // Return empty data instead of null
        }
        
        const response = await fetch(`/assets/json/calendar/${targetMonth}.json`);
        if (!response.ok) {
            console.error(`Failed to load calendar data for ${targetMonth}:`, response.status);
            return { events: [], books: {} };
        }
        
        const calendarData = await response.json();
        
        // Cache the loaded data
        monthlyDataCache.set(targetMonth, calendarData);
        
        return calendarData;
    } catch (error) {
        console.error(`Error loading calendar data for ${targetMonth}:`, error);
        return { events: [], books: {} };
    }
}

// ----- Internal helpers --------------------------------------------------

// Build or rebuild the EventCalendar instance
function initializeEventCalendar(events) {
    const calendarEl = document.getElementById('calendar');
    if (!calendarEl) return;

    // Destroy existing instance when re-initialising
    if (calendar) {
        try {
            EventCalendar.destroy(calendar);
        } catch (e) {
            // Some versions expose a destroy() method on the instance instead – try that as well
            if (typeof calendar.destroy === 'function') {
                calendar.destroy();
            }
        }
    }

    // Transform raw JSON events into EventCalendar compatible structure
    const mapEvents = evts => evts.map(ev => {
        const book = currentBooks[ev.book_id] || {};
        return {
            id: ev.book_id,
            title: book.title || 'Unknown Book',
            start: ev.start,
            end: ev.end || ev.start,
            allDay: true,
            backgroundColor: book.color || getEventColor(ev),
            borderColor: book.color || getEventColor(ev),
            textColor: '#ffffff',
            extendedProps: {
                ...ev,
                book_title: book.title || 'Unknown Book',
                authors: book.authors || [],
                book_path: book.book_path,
                book_cover: book.book_cover,
                color: book.color,
                md5: ev.book_id
            }
        };
    });

    calendar = EventCalendar.create(calendarEl, {
        view: 'dayGridMonth',
        headerToolbar: false,
        height: 'auto',
        firstDay: 1, // Monday
        displayEventEnd: false,
        editable: false,
        eventStartEditable: false,
        eventDurationEditable: false,
        events: mapEvents(events),
        eventClick: info => showEventModal(info.event.title, info.event.extendedProps),
        dateClick: info => console.debug('Date clicked:', info.dateStr),
        datesSet: info => {
            const viewTitle = info.view.title;
            const currentMonthDate = info.view.currentStart;
            
            updateCalendarTitleDirect(viewTitle);
            updateMonthlyStats(currentMonthDate);
            
            // Load data for the new month if it's different from current data
            const newMonth = `${currentMonthDate.getFullYear()}-${String(currentMonthDate.getMonth() + 1).padStart(2, '0')}`;
            updateDisplayedMonth(newMonth);
            
            // Scroll current day into view if needed
            setTimeout(() => scrollCurrentDayIntoView(), 100);
        }
    });
}

// Update the custom toolbar title (Month YYYY)
function updateCalendarTitle(currentDate) {
    const titleEl = document.getElementById('calendarTitle');
    if (!titleEl || !currentDate) return;

    const monthNames = [
        'January', 'February', 'March', 'April', 'May', 'June',
        'July', 'August', 'September', 'October', 'November', 'December'
    ];

    const month = currentDate.getMonth();
    const year = currentDate.getFullYear();
    titleEl.textContent = `${monthNames[month]} ${year}`;
}

// Update the calendar title directly with the provided title string
function updateCalendarTitleDirect(title) {
    if (!title) return;
    
    // Update mobile h1 title
    const mobileTitle = document.querySelector('.md\\:hidden h1');
    if (mobileTitle) {
        mobileTitle.textContent = title;
    }
    
    // Update desktop h2 title
    const desktopTitle = document.querySelector('.hidden.md\\:block');
    if (desktopTitle) {
        desktopTitle.textContent = title;
    }
}

// Deterministic colour hashing based on book title (+md5 when available)
function getEventColor(event) {
    const palette = [
        '#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6',
        '#06B6D4', '#84CC16', '#F97316', '#EC4899', '#6366F1'
    ];

    const book = currentBooks[event.book_id] || {};
    let hash = 0;
    const str = (book.title || '') + (event.book_id || '');
    for (let i = 0; i < str.length; i++) {
        hash = ((hash << 5) - hash) + str.charCodeAt(i);
        hash |= 0; // Convert to 32-bit int
    }

    return palette[Math.abs(hash) % palette.length];
}

// Show event details modal (animated)
function showEventModal(_title, event) {
    const modal = document.getElementById('eventModal');
    const modalCard = document.getElementById('modalCard');
    const modalTitle = document.getElementById('modalTitle');
    const viewBookBtn = document.getElementById('viewBookBtn');

    if (!modal || !modalCard || !modalTitle || !viewBookBtn) return;

    modalTitle.textContent = event.book_title;

    // Handle book cover visuals
    const coverContainer = document.getElementById('bookCoverContainer');
    const coverImg = document.getElementById('bookCoverImg');
    const coverPlaceholder = document.getElementById('bookCoverPlaceholder');

    if (coverImg && coverContainer && coverPlaceholder) {
        if (event.book_cover && event.book_cover.trim() !== '') {
            coverImg.src = event.book_cover;
            coverImg.onload = () => {
                coverContainer.classList.remove('hidden');
                coverPlaceholder.classList.add('hidden');
            };
            coverImg.onerror = () => {
                coverContainer.classList.add('hidden');
                coverPlaceholder.classList.remove('hidden');
            };
        } else {
            coverContainer.classList.add('hidden');
            coverPlaceholder.classList.remove('hidden');
        }
    }

    // Author / duration / pages read
    const authorEl = document.getElementById('modalAuthor');
    const readTimeEl = document.getElementById('modalReadTime');
    const pagesReadEl = document.getElementById('modalPagesRead');

    if (authorEl) {
        authorEl.textContent = event.authors?.length ? event.authors.join(', ') : 'Unknown Author';
    }
    if (readTimeEl) {
        readTimeEl.textContent = formatDuration(event.total_read_time);
    }
    if (pagesReadEl) {
        pagesReadEl.textContent = event.total_pages_read;
    }

    // View-book button setup
    if (event.book_path) {
        viewBookBtn.classList.remove('hidden');
        viewBookBtn.onclick = () => {
            hideModal(); // Ensure modal hidden immediately
            window.location.href = event.book_path;
        };
    } else {
        viewBookBtn.classList.add('hidden');
        viewBookBtn.onclick = null;
    }

    // Animate open
    modal.classList.remove('hidden');
    modal.classList.add('opacity-0');
    modalCard.classList.add('scale-95', 'opacity-0');
    modal.offsetHeight; // Force reflow
    requestAnimationFrame(() => {
        modal.classList.replace('opacity-0', 'opacity-100');
        modalCard.classList.remove('scale-95', 'opacity-0');
        modalCard.classList.add('scale-100', 'opacity-100');
    });
}

function hideModal() {
    const modal = document.getElementById('eventModal');
    const modalCard = document.getElementById('modalCard');
    if (!modal || !modalCard) return;

    modal.classList.replace('opacity-100', 'opacity-0');
    modalCard.classList.replace('scale-100', 'scale-95');
    modalCard.classList.replace('opacity-100', 'opacity-0');

    setTimeout(() => {
        modal.classList.add('hidden');
    }, 300);
}

function setupEventHandlers() {
    // Today
    document.getElementById('todayBtn')?.addEventListener('click', () => {
        if (calendar) {
            calendar.setOption('date', new Date());
        }
    });

    // Prev / next navigation
    document.getElementById('prevBtn')?.addEventListener('click', () => {
        if (calendar && typeof calendar.prev === 'function') {
            calendar.prev();
        }
    });
    document.getElementById('nextBtn')?.addEventListener('click', () => {
        if (calendar && typeof calendar.next === 'function') {
            calendar.next();
        }
    });

    // Modal close / backdrop click
    document.getElementById('closeModal')?.addEventListener('click', hideModal);
    document.getElementById('eventModal')?.addEventListener('click', (e) => {
        if (e.target === e.currentTarget) hideModal();
    });
}

// Update monthly statistics for the given month/year (preferring pre-calculated stats when available)
function updateMonthlyStats(currentDate) {
    if (!currentDate) return;

    const targetMonthKey = `${currentDate.getFullYear()}-${String(currentDate.getMonth() + 1).padStart(2, '0')}`;
    const monthData = monthlyDataCache.get(targetMonthKey);

    let booksRead = 0;
    let pagesRead = 0;
    let timeRead = 0;
    let daysPct = 0;

    if (monthData && monthData.stats) {
        ({ books_read: booksRead, pages_read: pagesRead, time_read: timeRead, days_read_pct: daysPct } = monthData.stats);
    }

    const booksEl = document.getElementById('monthlyBooks');
    const pagesEl = document.getElementById('monthlyPages');
    const timeEl = document.getElementById('monthlyTime');
    const daysPercentageEl = document.getElementById('monthlyDaysPercentage');

    if (booksEl) booksEl.textContent = booksRead;
    if (pagesEl) pagesEl.textContent = Number(pagesRead).toLocaleString();
    if (timeEl) timeEl.textContent = formatDuration(timeRead);
    if (daysPercentageEl) daysPercentageEl.textContent = `${daysPct}%`;
}

// Scroll the current day into view within the calendar container
function scrollCurrentDayIntoView() {
    const calendarContainer = document.querySelector('.calendar-container');
    const todayCell = document.querySelector('.ec-today');
    
    if (!calendarContainer || !todayCell) return;
    
    // Get the container's scroll width and visible width
    const containerRect = calendarContainer.getBoundingClientRect();
    const todayRect = todayCell.getBoundingClientRect();
    
    // Calculate if today is outside the visible area
    const containerLeft = containerRect.left;
    const containerRight = containerRect.right;
    const todayLeft = todayRect.left;
    const todayRight = todayRect.right;
    
    // If today is outside the visible area, scroll to center it
    if (todayLeft < containerLeft || todayRight > containerRight) {
        const todayCenter = todayLeft + (todayRect.width / 2);
        const containerCenter = containerLeft + (containerRect.width / 2);
        const scrollOffset = todayCenter - containerCenter;
        
        calendarContainer.scrollBy({
            left: scrollOffset,
            behavior: 'smooth'
        });
    }
}

// Convert seconds to a short human-readable string
function formatDuration(seconds) {
    if (!seconds || seconds < 0) return '0s';

    if (seconds < 60) {
        return `${seconds}s`;
    } else if (seconds < 3600) {
        const minutes = Math.floor(seconds / 60);
        const rem = seconds % 60;
        return rem ? `${minutes}m ${rem}s` : `${minutes}m`;
    }

    const hours = Math.floor(seconds / 3600);
    const remMins = Math.floor((seconds % 3600) / 60);
    return remMins ? `${hours}h ${remMins}m` : `${hours}h`;
}

// Helper to compute previous and next month keys (format: YYYY-MM) for a given month key
function getAdjacentMonths(monthKey) {
    if (!monthKey) return [null, null];
    const [yearStr, monthStr] = monthKey.split('-');
    const year = Number(yearStr);
    const monthIndex = Number(monthStr) - 1; // 0-based index

    const prevDate = new Date(year, monthIndex - 1, 1);
    const nextDate = new Date(year, monthIndex + 1, 1);

    const format = d => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
    return [format(prevDate), format(nextDate)];
}

// Recalculate currentEvents and currentBooks from everything in monthlyDataCache
function refreshAggregatedData() {
    currentEvents = [];
    currentBooks = {};

    for (const [, monthData] of monthlyDataCache) {
        currentEvents.push(...(monthData.events || []));
        Object.assign(currentBooks, monthData.books || {});
    }
} 