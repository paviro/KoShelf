// Calendar module: provides initializeCalendar() to bootstrap the reading calendar UI
// All logic is self-contained – nothing is written to or read from the global `window` object.
// Consumers can use:
//   import { initializeCalendar } from '/assets/js/calendar.js';
//   initializeCalendar();

let calendar;
let currentEvents = [];
let currentBooks = {};

// Track which year-month files we've already loaded to avoid duplicate fetches
const loadedMonths = new Set();
const loadedBooks = new Set();
let availableMonths = null; // Set once we load the index

// Helper: convert Date -> 'YYYY_MM'
const monthKey = dateObj => `${dateObj.getFullYear()}_${String(dateObj.getMonth() + 1).padStart(2, '0')}`;

// Fetch months index
async function loadMonthsIndex() {
    if (availableMonths) return;
    
    const resp = await fetch('/assets/json/calendar/available_months.json');
    if (!resp.ok) throw new Error('Failed loading months index');
    const arr = await resp.json();
    availableMonths = new Set(arr);
}

// Fetch events for a given month (YYYY_MM) if not yet loaded
async function loadMonthEvents(key) {
    if (loadedMonths.has(key)) return;
    if (availableMonths && availableMonths.size && !availableMonths.has(key)) {
        // No data for this month, mark as loaded and skip fetch to avoid 404
        loadedMonths.add(key);
        return;
    }
    const resp = await fetch(`/assets/json/calendar/events_${key}.json`);
    if (!resp.ok) throw new Error('Failed loading events for month ' + key);
    const data = await resp.json(); // expect array
    if (Array.isArray(data)) {
        // First load any missing book metadata
        const missingBookIds = Array.from(new Set(data.map(ev => ev.book_id).filter(id => !loadedBooks.has(id))));
        await Promise.all(missingBookIds.map(id => ensureBookLoaded(id)));

        currentEvents.push(...data);
    }
    loadedMonths.add(key);
}

// Load initial data (books + current month) before initializing UI
async function loadInitialCalendarData() {
    const today = new Date();
    await loadMonthsIndex();
    const key = monthKey(today);
    await loadMonthEvents(key);
}

// Exported entry point
export function initializeCalendar() {
    loadInitialCalendarData().then(() => {
        initializeEventCalendar(currentEvents, new Date());
        updateCalendarStats(currentEvents);
        setupEventHandlers();
    });
}

// Build or rebuild the EventCalendar instance
function initializeEventCalendar(events, initialDate = null) {
    const calendarEl = document.getElementById('calendar');
    if (!calendarEl) return;

    // Destroy existing instance when re-initialising
    if (calendar) {
        EventCalendar.destroy(calendar);
    }

    // Merge consecutive single-day events belonging to the same
    // book so that the calendar UI gets nice streak bars while our stats,
    // which rely on the unmerged `currentEvents`, stay exact.
    const displayEvents = mergeSingleDayEvents(events);

    // Transform raw JSON events into EventCalendar compatible structure
    const mapEvents = evts => evts.map(ev => {
        const book = currentBooks[ev.book_id] || {};
        return {
            id: ev.book_id,
            title: book.title || 'Unknown Book',
            start: ev.date,
            end: ev.end || ev.date,
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

    const calendarConfig = {
        view: 'dayGridMonth',
        headerToolbar: false,
        height: 'auto',
        firstDay: 1, // Monday
        displayEventEnd: false,
        editable: false,
        eventStartEditable: false,
        eventDurationEditable: false,
        events: mapEvents(displayEvents),
        eventClick: info => showEventModal(info.event.title, info.event.extendedProps),
        dateClick: info => console.debug('Date clicked:', info.dateStr),
        datesSet: info => {
            const viewTitle = info.view.title;
            const currentMonthDate = info.view.currentStart;
            
            updateCalendarTitleDirect(viewTitle);
            updateMonthlyStats(currentMonthDate);
            
            // Lazy-load data for the newly visible month
            ensureMonthLoaded(currentMonthDate);
            
            // Scroll current day into view if needed
            setTimeout(() => scrollCurrentDayIntoView(), 100);
        }
    };

    if (initialDate) {
        calendarConfig.date = initialDate;
    }

    calendar = EventCalendar.create(calendarEl, calendarConfig);
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
        const eventStart = new Date(event.date);
        const eventEnd = event.end ? new Date(event.end) : null; // end is exclusive when provided

        // Quick rejection when the whole event is outside the target month/year
        if (eventEnd) {
            // Event ends (exclusive) before the target month or starts after it – skip
            if (eventEnd.getFullYear() < targetYear || (eventEnd.getFullYear() === targetYear && eventEnd.getMonth() < targetMonth)) {
                return false;
            }
            if (eventStart.getFullYear() > targetYear || (eventStart.getFullYear() === targetYear && eventStart.getMonth() > targetMonth)) {
                return false;
            }
            return true;
        }

        return eventStart.getMonth() === targetMonth && eventStart.getFullYear() === targetYear;
    });
    
    // Calculate statistics (taking multi-day events into account)
    const uniqueBooks = new Set();
    const uniqueDates = new Set();
    let totalPages = 0;
    let totalTime = 0;

    const oneDayMs = 24 * 60 * 60 * 1000;

    monthlyEvents.forEach(event => {
        // Count unique books (using book_id)
        uniqueBooks.add(event.book_id);

        const startDate = new Date(event.date);
        // `end` is exclusive per EventCalendar / FullCalendar conventions. If absent, treat as single-day event.
        const endExclusive = event.end ? new Date(event.end) : new Date(startDate.getTime() + oneDayMs);

        // Walk through each day the event covers (within the target month/year)
        for (let d = new Date(startDate); d < endExclusive; d = new Date(d.getTime() + oneDayMs)) {
            if (d.getFullYear() === targetYear && d.getMonth() === targetMonth) {
                const dateString = d.toISOString().split('T')[0];
                uniqueDates.add(dateString);
            }
        }

        // Sum pages and time (counted once per event)
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

// Ensure data for a month is loaded (called when user navigates)
async function ensureMonthLoaded(dateObj) {
    const key = monthKey(dateObj);
    if (loadedMonths.has(key)) return;

    // Disable nav buttons while loading to prevent rapid clicks causing race conditions
    toggleNavButtons(true);

    await loadMonthEvents(key);

    // Re-enable buttons
    toggleNavButtons(false);

    // Rebuild calendar while preserving the currently requested month
    initializeEventCalendar(currentEvents, dateObj);
}

function toggleNavButtons(disabled) {
    const btnIds = ['prevBtn', 'nextBtn'];
    btnIds.forEach(id => {
        const el = document.getElementById(id);
        if (!el) return;
        if (disabled) {
            el.setAttribute('disabled', 'disabled');
            el.classList.add('opacity-50', 'pointer-events-none');
        } else {
            el.removeAttribute('disabled');
            el.classList.remove('opacity-50', 'pointer-events-none');
        }
    });
}

// ----- Internal helpers --------------------------------------------------

/**
 * Merge consecutive single-day events that belong to the same book into
 * multi-day span events. This is a visual optimisation for the calendar UI.
 * The original `currentEvents` array (single-day events) is left untouched
 * and continues to power statistics calculations.
 *
 * Each input event is expected to:
 *   – have a `date` field (yyyy-mm-dd) and be treated as an all-day event
 *   – carry per-day totals in `total_read_time` and `total_pages_read`
 *
 * The merged output events get an `end` field (exclusive) when the span is
 * longer than one day so EventCalendar can render a bar across days.
 */
function mergeSingleDayEvents(events) {
    if (!Array.isArray(events) || events.length === 0) return [];

    const sorted = [...events].sort((a, b) => {
        if (a.book_id === b.book_id) {
            return new Date(a.date) - new Date(b.date);
        }
        return a.book_id.localeCompare(b.book_id);
    });

    const merged = [];
    const oneDayMs = 24 * 60 * 60 * 1000;

    for (const ev of sorted) {
        const last = merged[merged.length - 1];

        if (last && last.book_id === ev.book_id) {
            const lastEndDate = new Date(last.end ? last.end : (new Date(last.date).getTime() + oneDayMs));
            const currentDate = new Date(ev.date);

            if (currentDate.getTime() === lastEndDate.getTime()) {
                // Extend span and totals
                last.total_read_time = (last.total_read_time || 0) + (ev.total_read_time || 0);
                last.total_pages_read = (last.total_pages_read || 0) + (ev.total_pages_read || 0);

                const newEnd = new Date(currentDate.getTime() + oneDayMs);
                last.end = newEnd.toISOString().split('T')[0];
                continue;
            }
        }

        merged.push({ ...ev });
    }

    return merged;
}

// Ensure a particular book metadata JSON is loaded
async function ensureBookLoaded(bookId) {
    if (loadedBooks.has(bookId)) return;
    try {
        const resp = await fetch(`/assets/json/calendar/books/${bookId}.json`);
        if (!resp.ok) {
            console.warn('No metadata for book', bookId);
            loadedBooks.add(bookId);
            return;
        }
        const bookData = await resp.json();
        currentBooks[bookId] = bookData;
        loadedBooks.add(bookId);
    } catch (err) {
        console.error('Error loading book metadata', bookId, err);
        loadedBooks.add(bookId);
    }
} 