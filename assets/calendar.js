// Calendar module: provides initializeCalendar() to bootstrap the reading calendar UI
// All logic is self-contained – nothing is written to or read from the global `window` object.
// Consumers can use:
//   import { initializeCalendar } from '/assets/js/calendar.js';
//   initializeCalendar();

let calendar;
let currentEvents = [];
let currentBooks = {};

// Exported entry point
export function initializeCalendar() {
    // Load calendar data from JSON
    loadCalendarData().then(calendarData => {
        if (calendarData) {
            currentEvents = calendarData.events || [];
            currentBooks = calendarData.books || {};

            initializeEventCalendar(currentEvents);

            // Populate statistics widgets
            updateCalendarStats(currentEvents);

            // Wire up DOM interaction handlers (today / prev / next / modal)
            setupEventHandlers();
        }
    });
}

// Load calendar data from JSON file
async function loadCalendarData() {
    try {
        const response = await fetch('/assets/json/calendar_data.json');
        if (!response.ok) {
            console.error('Failed to load calendar data:', response.status);
            return null;
        }
        return await response.json();
    } catch (error) {
        console.error('Error loading calendar data:', error);
        return null;
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
        events: mapEvents(events),
        eventClick: info => showEventModal(info.event.title, info.event.extendedProps),
        dateClick: info => console.debug('Date clicked:', info.dateStr),
        datesSet: info => {
            const viewTitle = info.view.title;
            const currentMonthDate = info.view.currentStart;
            
            updateCalendarTitleDirect(viewTitle);
            updateMonthlyStats(currentMonthDate);
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
        modal.classList.remove('opacity-0');
        modalCard.classList.remove('scale-95', 'opacity-0');
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

// Populate quick statistics boxes (legacy function - kept for compatibility)
function updateCalendarStats(events) {
    const now = new Date();
    updateMonthlyStats(now);
}

// Update monthly statistics for the given month/year
function updateMonthlyStats(currentDate) {
    if (!currentDate || !currentEvents) return;
    
    const targetMonth = currentDate.getMonth();
    const targetYear = currentDate.getFullYear();
    
    // Filter events for the target month/year
    const monthlyEvents = currentEvents.filter(event => {
        const eventDate = new Date(event.start);
        return eventDate.getMonth() === targetMonth && eventDate.getFullYear() === targetYear;
    });
    
    // Calculate statistics
    const uniqueBooks = new Set();
    const uniqueDates = new Set();
    let totalPages = 0;
    let totalTime = 0;
    
    monthlyEvents.forEach(event => {
        // Count unique books (using book_id)
        uniqueBooks.add(event.book_id);
        
        // Count unique dates (format as YYYY-MM-DD)
        const eventDate = new Date(event.start);
        const dateString = eventDate.toISOString().split('T')[0];
        uniqueDates.add(dateString);
        
        // Sum pages and time
        totalPages += event.total_pages_read || 0;
        totalTime += event.total_read_time || 0;
    });
    
    // Calculate days read percentage
    const daysInMonth = new Date(targetYear, targetMonth + 1, 0).getDate();
    const daysReadPercentage = Math.round((uniqueDates.size / daysInMonth) * 100);
    
    // Update DOM elements
    const booksEl = document.getElementById('monthlyBooks');
    const pagesEl = document.getElementById('monthlyPages');
    const timeEl = document.getElementById('monthlyTime');
    const daysPercentageEl = document.getElementById('monthlyDaysPercentage');
    
    if (booksEl) booksEl.textContent = uniqueBooks.size;
    if (pagesEl) pagesEl.textContent = totalPages.toLocaleString();
    if (timeEl) timeEl.textContent = formatDuration(totalTime);
    if (daysPercentageEl) daysPercentageEl.textContent = `${daysReadPercentage}%`;
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