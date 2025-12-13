// Calendar module: provides initializeCalendar() to bootstrap the reading calendar UI
// All logic is self-contained – nothing is written to or read from the global `window` object.

import { showModal, hideModal, setupModalCloseHandlers } from './modal-utils.js';
import { translation } from './i18n.js';

type ContentFilter = 'all' | 'book' | 'comic';
type ContentType = 'book' | 'comic';

// Type declarations for EventCalendar library
declare const EventCalendar: {
    create(el: HTMLElement, options: EventCalendarOptions): EventCalendarInstance;
    destroy(calendar: EventCalendarInstance): void;
};

interface EventCalendarOptions {
    view: string;
    headerToolbar: boolean;
    height: string;
    locale: string;
    firstDay: number;
    displayEventEnd: boolean;
    editable: boolean;
    eventStartEditable: boolean;
    eventDurationEditable: boolean;
    events: CalendarEvent[];
    eventClick: (info: EventClickInfo) => void;
    dateClick: (info: DateClickInfo) => void;
    datesSet: (info: DatesSetInfo) => void;
}

interface EventCalendarInstance {
    setOption<K extends keyof EventCalendarOptions>(key: K, value: EventCalendarOptions[K]): void;
    prev(): void;
    next(): void;
    destroy?(): void;
}

interface EventClickInfo {
    event: {
        title: string;
        extendedProps: EventExtendedProps;
    };
}

interface DateClickInfo {
    dateStr: string;
}

interface DatesSetInfo {
    view: {
        title: string;
        currentStart: Date;
    };
}

interface CalendarEvent {
    id: string;
    title: string;
    start: string;
    end: string;
    allDay: boolean;
    backgroundColor: string;
    borderColor: string;
    textColor: string;
    extendedProps: EventExtendedProps;
}

interface EventExtendedProps {
    book_id: string;
    start: string;
    end?: string;
    total_read_time: number;
    total_pages_read: number;
    book_title: string;
    authors: string[];
    book_path?: string;
    book_cover?: string;
    color?: string;
    content_type: ContentType;
    md5: string;
}

interface RawEvent {
    book_id: string;
    start: string;
    end?: string;
    total_read_time: number;
    total_pages_read: number;
}

interface BookInfo {
    title?: string;
    authors?: string[];
    book_path?: string;
    book_cover?: string;
    color?: string;
    content_type?: ContentType;
}

interface MonthlyStats {
    books_read: number;
    pages_read: number;
    time_read: number;
    days_read_pct: number;
}

interface MonthData {
    events: RawEvent[];
    books: Record<string, BookInfo>;
    stats?: MonthlyStats;
    stats_books?: MonthlyStats;
    stats_comics?: MonthlyStats;
}

let calendar: EventCalendarInstance | null = null;
let currentEvents: RawEvent[] = [];
let currentBooks: Record<string, BookInfo> = {};
const monthlyDataCache = new Map<string, MonthData>(); // Cache for monthly data: month -> {events, books}
let availableMonths: string[] = []; // List of months that have data
let currentDisplayedMonth: string | null = null;
let currentContentFilter: ContentFilter = loadInitialFilter();

function loadInitialFilter(): ContentFilter {
    try {
        const saved = localStorage.getItem('koshelf_calendar_filter');
        if (saved === 'all' || saved === 'book' || saved === 'comic') return saved;
    } catch {
        // ignore
    }
    return 'all';
}

// Exported entry point
export async function initializeCalendar(): Promise<void> {
    // Load translations first
    await translation.init();

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

            initializeEventCalendar(getFilteredRawEvents(currentEvents));

            // Populate statistics widgets for the initial month
            updateMonthlyStats(now);

            // Wire up DOM interaction handlers (today / prev / next / modal)
            setupEventHandlers();
        })();
    });
}

// Load the list of available months
async function loadAvailableMonths(): Promise<void> {
    try {
        const response = await fetch('/assets/json/calendar/available_months.json');
        if (response.ok) {
            availableMonths = await response.json() as string[];
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
async function updateDisplayedMonth(targetMonth: string): Promise<void> {
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
            renderCalendarEvents();

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
async function fetchMonthData(targetMonth: string | null): Promise<MonthData> {
    try {
        // If no target month specified, use current month
        if (!targetMonth) {
            const now = new Date();
            targetMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
        }

        // Check if month data is already cached
        const cached = monthlyDataCache.get(targetMonth);
        if (cached) {
            return cached;
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

        const calendarData = await response.json() as MonthData;

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
function initializeEventCalendar(events: RawEvent[]): void {
    const calendarEl = document.getElementById('calendar');
    if (!calendarEl) return;

    // Destroy existing instance when re-initialising
    if (calendar) {
        try {
            EventCalendar.destroy(calendar);
        } catch {
            // Some versions expose a destroy() method on the instance instead – try that as well
            if (typeof calendar.destroy === 'function') {
                calendar.destroy();
            }
        }
    }

    calendar = EventCalendar.create(calendarEl, {
        view: 'dayGridMonth',
        headerToolbar: false,
        height: 'auto',
        locale: translation.getLanguage(),
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

            // Update Today button disabled state
            updateTodayButtonState(currentMonthDate);

            // Scroll current day into view if needed
            setTimeout(() => scrollCurrentDayIntoView(), 100);
        }
    });

    // Set initial visual state of filter buttons
    syncFilterButtons();
}

function getEventContentType(ev: RawEvent): ContentType {
    const book = currentBooks[ev.book_id];
    return book?.content_type === 'comic' ? 'comic' : 'book';
}

function getFilteredRawEvents(evts: RawEvent[]): RawEvent[] {
    if (currentContentFilter === 'all') return evts;
    return evts.filter(ev => getEventContentType(ev) === currentContentFilter);
}

function mapEvents(evts: RawEvent[]): CalendarEvent[] {
    return evts.map(ev => {
        const book = currentBooks[ev.book_id] || {};
        const content_type: ContentType = (book.content_type === 'comic' ? 'comic' : 'book');
        return {
            id: ev.book_id,
            title: book.title || translation.get('unknown-book'),
            start: ev.start,
            end: ev.end || ev.start,
            allDay: true,
            backgroundColor: book.color || getEventColor(ev),
            borderColor: book.color || getEventColor(ev),
            textColor: '#ffffff',
            extendedProps: {
                ...ev,
                book_title: book.title || translation.get('unknown-book'),
                authors: book.authors || [],
                book_path: book.book_path,
                book_cover: book.book_cover,
                color: book.color,
                content_type,
                md5: ev.book_id
            }
        };
    });
}

function renderCalendarEvents(): void {
    if (!calendar) return;
    calendar.setOption('events', mapEvents(getFilteredRawEvents(currentEvents)));
}

function setContentFilter(filter: ContentFilter): void {
    currentContentFilter = filter;
    try {
        localStorage.setItem('koshelf_calendar_filter', filter);
    } catch {
        // ignore
    }

    syncFilterButtons();
    renderCalendarEvents();

    // Close dropdown if open
    const dropdown = document.getElementById('calendarFilterDropdown') as HTMLDetailsElement | null;
    if (dropdown) dropdown.open = false;

    // Refresh stats for the currently displayed month (if we know it)
    if (currentDisplayedMonth) {
        const [yr, mo] = currentDisplayedMonth.split('-');
        const statsDate = new Date(Number(yr), Number(mo) - 1, 1);
        updateMonthlyStats(statsDate);
    } else {
        updateMonthlyStats(new Date());
    }
}

function syncFilterButtons(): void {
    const labelEl = document.getElementById('calendarFilterLabel');
    if (labelEl) {
        if (currentContentFilter === 'book') {
            labelEl.textContent = translation.get('books');
        } else if (currentContentFilter === 'comic') {
            labelEl.textContent = translation.get('comics');
        } else {
            labelEl.textContent = translation.get('filter.all');
        }
    }
}

// Update the calendar title directly with the provided title string
function updateCalendarTitleDirect(title: string): void {
    if (!title) return;

    // Update mobile h1 title
    const mobileTitle = document.querySelector('.lg\\:hidden h1');
    if (mobileTitle) {
        mobileTitle.textContent = title;
    }

    // Update desktop h2 title
    const desktopTitle = document.querySelector('.hidden.lg\\:block');
    if (desktopTitle) {
        desktopTitle.textContent = title;
    }
}

// Deterministic colour hashing based on book title (+md5 when available)
function getEventColor(event: RawEvent): string {
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
function showEventModal(_title: string, event: EventExtendedProps): void {
    const modal = document.getElementById('eventModal');
    const modalCard = document.getElementById('modalCard');
    const modalTitle = document.getElementById('modalTitle');
    const viewBookBtn = document.getElementById('viewBookBtn') as HTMLButtonElement | null;

    if (!modal || !modalCard || !modalTitle || !viewBookBtn) return;

    modalTitle.textContent = event.book_title;

    // Handle book cover visuals
    const coverContainer = document.getElementById('bookCoverContainer');
    const coverImg = document.getElementById('bookCoverImg') as HTMLImageElement | null;
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
        authorEl.textContent = event.authors?.length ? event.authors.join(', ') : translation.get('unknown-author');
    }
    if (readTimeEl) {
        readTimeEl.textContent = formatDuration(event.total_read_time);
    }
    if (pagesReadEl) {
        pagesReadEl.textContent = String(event.total_pages_read);
    }

    // View-book button setup
    if (event.book_path) {
        viewBookBtn.classList.remove('hidden');
        viewBookBtn.onclick = () => {
            hideEventModal(); // Ensure modal hidden immediately
            window.location.href = event.book_path!;
        };
    } else {
        viewBookBtn.classList.add('hidden');
        viewBookBtn.onclick = null;
    }

    // Animate open using shared utility
    showModal(modal, modalCard);
}

// Helper to hide the event modal using the shared utility
function hideEventModal(): void {
    const modal = document.getElementById('eventModal');
    const modalCard = document.getElementById('modalCard');
    hideModal(modal, modalCard);
}

function setupEventHandlers(): void {
    // Filter buttons
    document.querySelectorAll<HTMLElement>('.calendar-filter-btn').forEach(el => {
        el.addEventListener('click', (e) => {
            e.preventDefault();
            const next = ((el as HTMLElement).dataset.calendarFilter || 'all') as ContentFilter;
            setContentFilter(next);
        });
    });

    // Today
    const todayBtn = document.getElementById('todayBtn') as HTMLButtonElement | null;
    todayBtn?.addEventListener('click', () => {
        if (!calendar || !todayBtn || todayBtn.disabled) return;

        // EventCalendar doesn't provide a stable public "go to today" API in this build.
        // Re-creating the instance reliably resets the view to the current month/day.
        initializeEventCalendar(getFilteredRawEvents(currentEvents));
        setTimeout(() => scrollCurrentDayIntoView(), 100);
    });

    // Set initial state of Today button
    updateTodayButtonState(new Date());

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

    // Modal close handlers using shared utility
    const modal = document.getElementById('eventModal');
    const modalCard = document.getElementById('modalCard');
    const closeBtn = document.getElementById('closeModal');
    setupModalCloseHandlers(modal, modalCard, closeBtn);
}

// Update Today button disabled state based on whether we're viewing the current month
function updateTodayButtonState(displayedDate: Date): void {
    const todayBtn = document.getElementById('todayBtn') as HTMLButtonElement | null;
    if (!todayBtn) return;

    const now = new Date();
    const isCurrentMonth =
        displayedDate.getFullYear() === now.getFullYear() &&
        displayedDate.getMonth() === now.getMonth();

    todayBtn.disabled = isCurrentMonth;

    if (isCurrentMonth) {
        // Disabled styling - match recap page disabled buttons
        todayBtn.classList.remove('bg-primary-600', 'hover:bg-primary-700', 'text-white');
        todayBtn.classList.add('bg-gray-100', 'dark:bg-dark-800', 'text-gray-400', 'dark:text-dark-400', 'cursor-not-allowed');
    } else {
        // Enabled styling - primary color
        todayBtn.classList.add('bg-primary-600', 'hover:bg-primary-700', 'text-white');
        todayBtn.classList.remove('bg-gray-100', 'dark:bg-dark-800', 'text-gray-400', 'dark:text-dark-400', 'cursor-not-allowed');
    }
}

// Update monthly statistics for the given month/year (preferring pre-calculated stats when available)
function updateMonthlyStats(currentDate: Date): void {
    if (!currentDate) return;

    const targetMonthKey = `${currentDate.getFullYear()}-${String(currentDate.getMonth() + 1).padStart(2, '0')}`;
    const monthData = monthlyDataCache.get(targetMonthKey);

    let booksRead = 0;
    let pagesRead = 0;
    let timeRead = 0;
    let daysPct = 0;

    const pickStats = (): MonthlyStats | undefined => {
        if (!monthData) return undefined;
        if (currentContentFilter === 'book') return monthData.stats_books || monthData.stats;
        if (currentContentFilter === 'comic') return monthData.stats_comics || monthData.stats;
        return monthData.stats;
    };

    const stats = pickStats();
    if (stats) {
        ({ books_read: booksRead, pages_read: pagesRead, time_read: timeRead, days_read_pct: daysPct } = stats);
    }

    const booksEl = document.getElementById('monthlyBooks');
    const booksLabelEl = document.getElementById('monthlyBooksLabel');
    const pagesEl = document.getElementById('monthlyPages');
    const timeEl = document.getElementById('monthlyTime');
    const daysPercentageEl = document.getElementById('monthlyDaysPercentage');

    if (booksEl) booksEl.textContent = String(booksRead);
    if (booksLabelEl) {
        if (currentContentFilter === 'comic') {
            booksLabelEl.textContent = translation.get('comic-label', { count: booksRead });
        } else {
            booksLabelEl.textContent = translation.get('book-label', { count: booksRead });
        }
    }
    if (pagesEl) pagesEl.textContent = Number(pagesRead).toLocaleString();
    if (timeEl) timeEl.textContent = formatDuration(timeRead);
    if (daysPercentageEl) daysPercentageEl.textContent = `${daysPct}%`;
}

// Scroll the current day into view within the calendar container
function scrollCurrentDayIntoView(): void {
    const calendarContainer = document.querySelector<HTMLElement>('.calendar-container');
    const todayCell = document.querySelector<HTMLElement>('.ec-today');

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
function formatDuration(seconds: number): string {
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
function getAdjacentMonths(monthKey: string): [string | null, string | null] {
    if (!monthKey) return [null, null];
    const [yearStr, monthStr] = monthKey.split('-');
    const year = Number(yearStr);
    const monthIndex = Number(monthStr) - 1; // 0-based index

    const prevDate = new Date(year, monthIndex - 1, 1);
    const nextDate = new Date(year, monthIndex + 1, 1);

    const format = (d: Date): string => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
    return [format(prevDate), format(nextDate)];
}

// Recalculate currentEvents and currentBooks from everything in monthlyDataCache
function refreshAggregatedData(): void {
    currentEvents = [];
    currentBooks = {};

    const seenKeys = new Set<string>();

    for (const [, monthData] of monthlyDataCache) {
        for (const ev of monthData.events || []) {
            const key = `${ev.book_id}|${ev.start}|${ev.end || ''}`;
            if (!seenKeys.has(key)) {
                seenKeys.add(key);
                currentEvents.push(ev);
            }
        }
        Object.assign(currentBooks, monthData.books || {});
    }
}
