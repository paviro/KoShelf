// Calendar module: provides initializeCalendar() to bootstrap the reading calendar UI
// All logic is self-contained – nothing is written to or read from the global `window` object.

// Event Calendar (module API)
import { createCalendar, destroyCalendar, DayGrid } from '@event-calendar/core';
import type { Calendar } from '@event-calendar/core';

import { showModal, hideModal, setupModalCloseHandlers } from '../components/modal-utils.js';
import { translation } from '../shared/i18n.js';

type ContentFilter = 'all' | 'book' | 'comic';
type ContentType = 'book' | 'comic';

function byId<T extends HTMLElement>(id: string): T | null {
    return document.getElementById(id) as T | null;
}

function parseContentFilter(value: string | undefined | null): ContentFilter {
    if (value === 'all' || value === 'book' || value === 'comic') return value;
    return 'all';
}

function monthKey(date: Date): string {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
}

interface EventExtendedProps extends Record<string, unknown> {
    item_id: string;
    start: string;
    end?: string;
    total_read_time: number;
    total_pages_read: number;
    book_title: string;
    authors: string[];
    item_path?: string;
    item_cover?: string;
    color?: string;
    content_type: ContentType;
    md5: string;
}

interface RawEvent {
    item_id: string;
    start: string;
    end?: string;
    total_read_time: number;
    total_pages_read: number;
}

interface BookInfo {
    title?: string;
    authors?: string[];
    item_path?: string;
    item_cover?: string;
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

let calendar: Calendar | null = null;
let currentEvents: RawEvent[] = [];
let currentBooks: Record<string, BookInfo> = {};
const monthlyDataCache = new Map<string, MonthData>(); // Cache for monthly data: month -> {events, books}
let availableMonths: string[] = []; // List of months that have data
let currentDisplayedMonth: string | null = null;
let currentContentFilter: ContentFilter = loadInitialFilter();

// Month/Year picker state
let currentPickerYear: number = new Date().getFullYear();
let _currentPickerMonth: number = new Date().getMonth();
let yearPickerStartYear: number = new Date().getFullYear() - 4; // Show 9 years (4 before, current, 4 after)

// Cached modal element references (initialized in setupDatePickerHandlers)
let monthPickerModal: HTMLElement | null = null;
let monthPickerCard: HTMLElement | null = null;
let yearPickerModal: HTMLElement | null = null;
let yearPickerCard: HTMLElement | null = null;

function loadInitialFilter(): ContentFilter {
    try {
        const saved = localStorage.getItem('koshelf_calendar_filter');
        return parseContentFilter(saved);
    } catch {
        // ignore
    }
    return 'all';
}

// Exported entry point
export async function initializeCalendar(): Promise<void> {
    // Load translations first
    await translation.init();

    // Load the list of available months first (best-effort; it can fail and we still continue).
    await loadAvailableMonths();

    // Load calendar data for current month and its neighbours
    const now = new Date();
    const currentMonth = monthKey(now);
    const [prevMonth, nextMonth] = getAdjacentMonths(currentMonth);

    // Load current, previous and next month (if they exist)
    await Promise.all([
        fetchMonthData(prevMonth),
        fetchMonthData(currentMonth),
        fetchMonthData(nextMonth),
    ]);

    currentDisplayedMonth = currentMonth;
    refreshAggregatedData();

    initializeEventCalendar(getFilteredRawEvents(currentEvents));

    // Populate statistics widgets for the initial month
    updateMonthlyStats(now);

    // Wire up DOM interaction handlers (today / prev / next / modal)
    setupEventHandlers();
}

// When this module is loaded as a page entry bundle, auto-initialize.
// (Still exported for tests / potential future reuse.)
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        initializeCalendar().catch((e) => console.error('Failed to initialize calendar:', e));
    });
} else {
    initializeCalendar().catch((e) => console.error('Failed to initialize calendar:', e));
}

// Load the list of available months
async function loadAvailableMonths(): Promise<void> {
    try {
        const response = await fetch('/assets/json/calendar/available_months.json');
        if (response.ok) {
            availableMonths = (await response.json()) as string[];
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
            fetchMonthData(nextMonth),
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

        const calendarData = (await response.json()) as MonthData;

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
    const calendarEl = byId<HTMLElement>('calendar');
    if (!calendarEl) return;

    // Destroy existing instance when re-initialising
    if (calendar) {
        // destroyCalendar returns a Promise; we don't need to await for our use.
        void destroyCalendar(calendar);
    }

    calendar = createCalendar(calendarEl, [DayGrid], {
        view: 'dayGridMonth',
        height: 'auto',
        locale: translation.getLanguage(),
        firstDay: 1, // Monday
        displayEventEnd: false,
        editable: false,
        eventStartEditable: false,
        eventDurationEditable: false,
        events: mapEvents(events),
        eventClick: (info) => {
            // event.title is `Calendar.Content` (string | {html} | {domNodes}); we don't need it for our modal.
            showEventModal('', info.event.extendedProps as unknown as EventExtendedProps);
        },
        dateClick: (info) => console.debug('Date clicked:', info.dateStr),
        datesSet: (info) => {
            const currentMonthDate = info.view.currentStart;

            updateCalendarTitle(currentMonthDate);
            updateMonthlyStats(currentMonthDate);

            // Load data for the new month if it's different from current data
            const newMonth = `${currentMonthDate.getFullYear()}-${String(currentMonthDate.getMonth() + 1).padStart(2, '0')}`;
            updateDisplayedMonth(newMonth);

            // Update Today button disabled state
            updateTodayButtonState(currentMonthDate);

            // Scroll current day into view if needed
            setTimeout(() => scrollCurrentDayIntoView(), 100);
        },
    });

    // Set initial visual state of filter buttons
    syncFilterButtons();
}

function getEventContentType(ev: RawEvent): ContentType {
    const book = currentBooks[ev.item_id];
    return book?.content_type === 'comic' ? 'comic' : 'book';
}

function getFilteredRawEvents(evts: RawEvent[]): RawEvent[] {
    if (currentContentFilter === 'all') return evts;
    return evts.filter((ev) => getEventContentType(ev) === currentContentFilter);
}

function mapEvents(evts: RawEvent[]): Array<Calendar.EventInput> {
    return evts.map((ev) => {
        const book = currentBooks[ev.item_id] || {};
        const content_type: ContentType = book.content_type === 'comic' ? 'comic' : 'book';
        const extendedProps: EventExtendedProps = {
            ...ev,
            book_title: book.title || translation.get('unknown-book'),
            authors: book.authors || [],
            item_path: book.item_path,
            item_cover: book.item_cover,
            color: book.color,
            content_type,
            md5: ev.item_id,
        };

        const input: Calendar.EventInput = {
            id: ev.item_id,
            title: book.title || translation.get('unknown-book'),
            start: ev.start,
            end: ev.end || ev.start,
            allDay: true,
            backgroundColor: book.color || getEventColor(ev),
            textColor: '#ffffff',
            extendedProps,
        };

        return input;
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
    const dropdown = byId<HTMLDetailsElement>('calendarFilterDropdown');
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
    const filterIcon = document.getElementById('calendarFilterIcon');

    if (labelEl) {
        if (currentContentFilter === 'book') {
            labelEl.textContent = translation.get('books');
        } else if (currentContentFilter === 'comic') {
            labelEl.textContent = translation.get('comics');
        } else {
            labelEl.textContent = translation.get('filter.all');
        }
    }

    // Update filter icon color (gray when all, primary when filtered)
    if (filterIcon) {
        if (currentContentFilter === 'all') {
            filterIcon.classList.remove('text-primary-500');
            filterIcon.classList.add('text-gray-600', 'dark:text-gray-300');
        } else {
            filterIcon.classList.remove('text-gray-600', 'dark:text-gray-300');
            filterIcon.classList.add('text-primary-500');
        }
    }
}

// Update the calendar title using the displayed date
function updateCalendarTitle(date: Date): void {
    if (!date) return;

    const locale = translation.getLanguage() || 'en';
    const monthFormatter = new Intl.DateTimeFormat(locale, { month: 'long' });
    const yearFormatter = new Intl.DateTimeFormat(locale, { year: 'numeric' });

    const monthName = monthFormatter.format(date);
    const year = yearFormatter.format(date);

    // Update mobile buttons
    const mobileMonthBtn = document.getElementById('mobileMonthBtn');
    const mobileYearBtn = document.getElementById('mobileYearBtn');
    if (mobileMonthBtn) mobileMonthBtn.textContent = monthName;
    if (mobileYearBtn) mobileYearBtn.textContent = year;

    // Update desktop buttons
    const desktopMonthBtn = document.getElementById('desktopMonthBtn');
    const desktopYearBtn = document.getElementById('desktopYearBtn');
    if (desktopMonthBtn) desktopMonthBtn.textContent = monthName;
    if (desktopYearBtn) desktopYearBtn.textContent = year;
}

// Deterministic colour hashing based on book title (+md5 when available)
function getEventColor(event: RawEvent): string {
    const palette = [
        '#3B82F6',
        '#10B981',
        '#F59E0B',
        '#EF4444',
        '#8B5CF6',
        '#06B6D4',
        '#84CC16',
        '#F97316',
        '#EC4899',
        '#6366F1',
    ];

    const book = currentBooks[event.item_id] || {};
    let hash = 0;
    const str = (book.title || '') + (event.item_id || '');
    for (let i = 0; i < str.length; i++) {
        hash = (hash << 5) - hash + str.charCodeAt(i);
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
        if (event.item_cover && event.item_cover.trim() !== '') {
            coverImg.src = event.item_cover;
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
        authorEl.textContent = event.authors?.length
            ? event.authors.join(', ')
            : translation.get('unknown-author');
    }
    if (readTimeEl) {
        readTimeEl.textContent = formatDuration(event.total_read_time);
    }
    if (pagesReadEl) {
        pagesReadEl.textContent = String(event.total_pages_read);
    }

    // View-item button setup
    const itemPath = event.item_path;
    if (itemPath) {
        viewBookBtn.classList.remove('hidden');
        viewBookBtn.onclick = () => {
            hideEventModal(); // Ensure modal hidden immediately
            window.location.href = itemPath;
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
    document.querySelectorAll<HTMLElement>('.calendar-filter-btn').forEach((el) => {
        el.addEventListener('click', (e) => {
            e.preventDefault();
            const target = e.currentTarget as HTMLElement;
            const next = parseContentFilter(target.dataset.calendarFilter);
            setContentFilter(next);
        });
    });

    // Today
    const todayBtn = byId<HTMLButtonElement>('todayBtn');
    todayBtn?.addEventListener('click', () => {
        if (!calendar || !todayBtn || todayBtn.disabled) return;
        calendar.setOption('date', new Date());
        setTimeout(() => scrollCurrentDayIntoView(), 100);
    });

    // Set initial state of Today button
    updateTodayButtonState(new Date());

    // Prev / next navigation
    byId<HTMLElement>('prevBtn')?.addEventListener('click', () => {
        if (calendar && typeof calendar.prev === 'function') {
            calendar.prev();
        }
    });
    byId<HTMLElement>('nextBtn')?.addEventListener('click', () => {
        if (calendar && typeof calendar.next === 'function') {
            calendar.next();
        }
    });

    // Month/Year picker handlers
    setupDatePickerHandlers();

    // Modal close handlers using shared utility
    const modal = byId<HTMLElement>('eventModal');
    const modalCard = byId<HTMLElement>('modalCard');
    const closeBtn = byId<HTMLElement>('closeModal');
    setupModalCloseHandlers(modal, modalCard, closeBtn);
}

// Update Today button disabled state based on whether we're viewing the current month
function updateTodayButtonState(displayedDate: Date): void {
    const todayBtn = byId<HTMLButtonElement>('todayBtn');
    if (!todayBtn) return;

    const now = new Date();
    const isCurrentMonth =
        displayedDate.getFullYear() === now.getFullYear() &&
        displayedDate.getMonth() === now.getMonth();

    todayBtn.disabled = isCurrentMonth;

    if (isCurrentMonth) {
        // Disabled styling - match recap page disabled buttons
        todayBtn.classList.remove('bg-primary-600', 'hover:bg-primary-700', 'text-white');
        todayBtn.classList.add(
            'bg-gray-100',
            'dark:bg-dark-800',
            'text-gray-400',
            'dark:text-dark-400',
            'cursor-not-allowed',
        );
    } else {
        // Enabled styling - primary color
        todayBtn.classList.add('bg-primary-600', 'hover:bg-primary-700', 'text-white');
        todayBtn.classList.remove(
            'bg-gray-100',
            'dark:bg-dark-800',
            'text-gray-400',
            'dark:text-dark-400',
            'cursor-not-allowed',
        );
    }
}

// Update monthly statistics for the given month/year (preferring pre-calculated stats when available)
function updateMonthlyStats(currentDate: Date): void {
    const targetMonthKey = monthKey(currentDate);
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
        ({
            books_read: booksRead,
            pages_read: pagesRead,
            time_read: timeRead,
            days_read_pct: daysPct,
        } = stats);
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
        const todayCenter = todayLeft + todayRect.width / 2;
        const containerCenter = containerLeft + containerRect.width / 2;
        const scrollOffset = todayCenter - containerCenter;

        // Clamp desired scroll position to valid range to prevent overshooting
        const currentScroll = calendarContainer.scrollLeft;
        const maxScrollLeft = calendarContainer.scrollWidth - calendarContainer.clientWidth;
        const desiredScroll = Math.max(0, Math.min(currentScroll + scrollOffset, maxScrollLeft));
        const clampedOffset = desiredScroll - currentScroll;

        // Only scroll if there's a meaningful offset after clamping
        if (Math.abs(clampedOffset) > 1) {
            calendarContainer.scrollBy({
                left: clampedOffset,
                behavior: 'smooth',
            });
        }
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

    const format = (d: Date): string =>
        `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
    return [format(prevDate), format(nextDate)];
}

// Recalculate currentEvents and currentBooks from everything in monthlyDataCache
function refreshAggregatedData(): void {
    currentEvents = [];
    currentBooks = {};

    const seenKeys = new Set<string>();

    for (const [, monthData] of monthlyDataCache) {
        for (const ev of monthData.events || []) {
            const key = `${ev.item_id}|${ev.start}|${ev.end || ''}`;
            if (!seenKeys.has(key)) {
                seenKeys.add(key);
                currentEvents.push(ev);
            }
        }
        Object.assign(currentBooks, monthData.books || {});
    }
}

// ========== Month/Year Picker Functions ==========

function setupDatePickerHandlers(): void {
    // Cache modal element references
    monthPickerModal = document.getElementById('monthPickerModal');
    monthPickerCard = document.getElementById('monthPickerCard');
    yearPickerModal = document.getElementById('yearPickerModal');
    yearPickerCard = document.getElementById('yearPickerCard');

    // Month buttons click handlers (DRY)
    ['mobileMonthBtn', 'desktopMonthBtn'].forEach((id) =>
        document.getElementById(id)?.addEventListener('click', showMonthPicker),
    );
    ['mobileYearBtn', 'desktopYearBtn'].forEach((id) =>
        document.getElementById(id)?.addEventListener('click', showYearPicker),
    );

    // Use modal-utils for backdrop click and Escape key handling
    setupModalCloseHandlers(monthPickerModal, monthPickerCard);
    setupModalCloseHandlers(yearPickerModal, yearPickerCard);

    // Year picker navigation
    document.getElementById('yearPickerPrev')?.addEventListener('click', () => {
        yearPickerStartYear -= 9;
        populateYearPickerGrid();
    });
    document.getElementById('yearPickerNext')?.addEventListener('click', () => {
        yearPickerStartYear += 9;
        populateYearPickerGrid();
    });
}

function showMonthPicker(): void {
    // Set picker month/year to current displayed month
    if (currentDisplayedMonth) {
        const [yr, mo] = currentDisplayedMonth.split('-');
        currentPickerYear = Number(yr);
        _currentPickerMonth = Number(mo) - 1;
    }

    populateMonthPickerGrid();
    showModal(monthPickerModal, monthPickerCard);
}

function hideMonthPicker(): void {
    hideModal(monthPickerModal, monthPickerCard);
}

function showYearPicker(): void {
    // Set starting year range based on current displayed month
    if (currentDisplayedMonth) {
        const [yr] = currentDisplayedMonth.split('-');
        currentPickerYear = Number(yr);
        yearPickerStartYear = currentPickerYear - 4;
    }

    populateYearPickerGrid();
    showModal(yearPickerModal, yearPickerCard);
}

function hideYearPicker(): void {
    hideModal(yearPickerModal, yearPickerCard);
}

// Helper to create styled picker buttons
function createPickerButton(
    text: string,
    isActive: boolean,
    onClick: () => void,
): HTMLButtonElement {
    const btn = document.createElement('button');
    btn.className = `px-3 py-2 text-sm rounded-lg transition-colors ${isActive
        ? 'bg-primary-600 text-white'
        : 'hover:bg-gray-100 dark:hover:bg-dark-700 text-gray-700 dark:text-gray-300'
        }`;
    btn.textContent = text;
    btn.addEventListener('click', onClick);
    return btn;
}

function populateMonthPickerGrid(): void {
    const grid = document.getElementById('monthPickerGrid');
    if (!grid) return;

    grid.innerHTML = '';

    // Get localized month names
    const locale = translation.getLanguage() || 'en';
    const monthFormatter = new Intl.DateTimeFormat(locale, { month: 'short' });

    for (let i = 0; i < 12; i++) {
        const date = new Date(currentPickerYear, i, 1);
        const monthName = monthFormatter.format(date);
        const isCurrentMonth =
            currentDisplayedMonth === `${currentPickerYear}-${String(i + 1).padStart(2, '0')}`;

        const btn = createPickerButton(monthName, isCurrentMonth, () => {
            navigateToMonth(currentPickerYear, i);
            hideMonthPicker();
        });

        grid.appendChild(btn);
    }
}

function populateYearPickerGrid(): void {
    const grid = document.getElementById('yearPickerGrid');
    const title = document.getElementById('yearPickerTitle');
    if (!grid) return;

    grid.innerHTML = '';

    // Update title to show year range
    if (title) {
        title.textContent = `${yearPickerStartYear} - ${yearPickerStartYear + 8}`;
    }

    // Get current displayed year for highlighting
    let displayedYear = new Date().getFullYear();
    if (currentDisplayedMonth) {
        displayedYear = Number(currentDisplayedMonth.split('-')[0]);
    }

    for (let i = 0; i < 9; i++) {
        const year = yearPickerStartYear + i;
        const isCurrentYear = year === displayedYear;

        const btn = createPickerButton(String(year), isCurrentYear, () => {
            // Navigate to the same month but different year
            let month = new Date().getMonth();
            if (currentDisplayedMonth) {
                month = Number(currentDisplayedMonth.split('-')[1]) - 1;
            }
            navigateToMonth(year, month);
            hideYearPicker();
        });

        grid.appendChild(btn);
    }
}

function navigateToMonth(year: number, month: number): void {
    if (!calendar) return;

    // Use setOption('date') to jump directly to the target month
    // This is more efficient than looping through prev/next
    const targetDate = new Date(year, month, 1);
    calendar.setOption('date', targetDate);
}
