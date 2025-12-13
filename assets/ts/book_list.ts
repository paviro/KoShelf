// KoInsight - Reading Tracker Interface
import { LazyImageLoader } from './lazy-loading.js';
import { SectionToggle } from './section-toggle.js';
import { translation } from './i18n.js';
import { initBookCardTilt } from './tilt-effect.js';

declare global {
    interface Window {
        bookListSections: SectionToggle;
    }
}

interface BookCard extends HTMLElement {
    dataset: {
        title?: string;
        author?: string;
        series?: string;
        status?: string;
    };
}

document.addEventListener('DOMContentLoaded', async () => {
    // Load translations for dynamic aria-label updates
    await translation.init();
    const searchInput = document.getElementById('searchInput') as HTMLInputElement | null;
    const mobileSearchInput = document.getElementById('mobileSearchInput') as HTMLInputElement | null;
    const filterButtons = document.querySelectorAll<HTMLElement>('[data-filter]');
    const readingSection = document.querySelector<HTMLElement>('section:has(#readingContainer)');
    const completedSection = document.querySelector<HTMLElement>('section:has(#completedContainer)');
    const abandonedSection = document.querySelector<HTMLElement>('section:has(#abandonedContainer)');
    const unreadSection = document.querySelector<HTMLElement>('section:has(#unreadContainer)');
    const bookCards = document.querySelectorAll<BookCard>('.book-card');

    let currentFilter = 'all';

    // Initialize lazy loading
    const lazyLoader = new LazyImageLoader();
    lazyLoader.init();

    // Initialize section toggles
    const sectionToggle = new SectionToggle();

    // Initialize 3D tilt effect on book cards
    initBookCardTilt();

    // Make it globally available for debugging or external control
    window.bookListSections = sectionToggle;

    // Check for search query in URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const searchQuery = urlParams.get('search');

    if (searchQuery) {
        if (searchInput) searchInput.value = searchQuery;
        if (mobileSearchInput) mobileSearchInput.value = searchQuery;

        // Remove the search parameter from URL without refreshing the page
        const url = new URL(window.location.href);
        url.searchParams.delete('search');
        window.history.replaceState({}, document.title, url.toString());

        // Trigger the search
        setTimeout(() => {
            handleSearchInput(searchQuery);
        }, 100);
    }

    let lastSearchTerm = '';

    // Unified search handler
    function handleSearchInput(value: string): void {
        const searchTerm = value.toLowerCase().trim();
        // Restore persisted/default state when search is cleared
        if (!searchTerm && lastSearchTerm) {
            sectionToggle.restorePersistedOrDefault();
        }
        lastSearchTerm = searchTerm;
        filterBooks(searchTerm, currentFilter);
        // After filtering, expand/collapse sections based on visible books
        if (searchTerm) {
            // For each section, check if it has visible books
            ['reading', 'completed', 'abandoned', 'unread'].forEach(name => {
                const container = document.getElementById(name + 'Container');
                if (!container) return;
                const hasVisible = Array.from(container.children).some(
                    child => (child as HTMLElement).style.display !== 'none'
                );
                if (hasVisible) {
                    sectionToggle.show(name, { persist: false });
                } else {
                    sectionToggle.hide(name, { persist: false });
                }
            });
        }
    }

    // Attach input listeners to both inputs (if they exist)
    [searchInput, mobileSearchInput].forEach(inp => {
        if (!inp) return;
        inp.addEventListener('input', function (this: HTMLInputElement) {
            // keep values in sync
            if (inp === searchInput && mobileSearchInput) {
                mobileSearchInput.value = inp.value;
            }
            if (inp === mobileSearchInput && searchInput) {
                searchInput.value = inp.value;
            }
            handleSearchInput(inp.value);
        });
    });

    // Filter functionality
    filterButtons.forEach(button => {
        button.addEventListener('click', function (this: HTMLElement) {
            // Update filter button states
            filterButtons.forEach(btn => btn.classList.remove('filter-button-active'));
            this.classList.add('filter-button-active');

            currentFilter = this.dataset.filter || 'all';
            const searchTerm = searchInput ? searchInput.value.toLowerCase().trim() : '';
            filterBooks(searchTerm, currentFilter);
        });
    });

    function filterBooks(searchTerm: string, filter: string): void {
        let readingVisible = 0;
        let completedVisible = 0;
        let abandonedVisible = 0;
        let unreadVisible = 0;

        bookCards.forEach(card => {
            const title = (card.dataset.title || '').toLowerCase();
            const author = (card.dataset.author || '').toLowerCase();
            const series = (card.dataset.series || '').toLowerCase();
            const status = card.dataset.status || '';

            // Check search match
            const matchesSearch = !searchTerm ||
                title.includes(searchTerm) ||
                author.includes(searchTerm) ||
                series.includes(searchTerm);

            // Check filter match
            const matchesFilter = filter === 'all' ||
                (filter === 'reading' && status === 'reading') ||
                (filter === 'completed' && status === 'completed') ||
                (filter === 'abandoned' && status === 'abandoned') ||
                (filter === 'unread' && status === 'unread');

            // Show/hide card with animation
            if (matchesSearch && matchesFilter) {
                card.style.display = 'block';
                card.style.opacity = '0';
                card.style.transform = 'translateY(20px)';
                requestAnimationFrame(() => {
                    card.style.transition = 'opacity 0.3s ease, transform 0.3s ease';
                    card.style.opacity = '1';
                    card.style.transform = 'translateY(0)';
                });

                // Count visible books by status
                if (status === 'reading') readingVisible++;
                if (status === 'completed') completedVisible++;
                if (status === 'abandoned') abandonedVisible++;
                if (status === 'unread') unreadVisible++;
            } else {
                card.style.display = 'none';
            }
        });

        // Show/hide sections based on content and filter
        if (readingSection) {
            const shouldShowReading = readingVisible > 0 && (filter === 'all' || filter === 'reading');
            readingSection.style.display = shouldShowReading ? 'block' : 'none';
        }

        if (completedSection) {
            const shouldShowCompleted = completedVisible > 0 && (filter === 'all' || filter === 'completed');
            completedSection.style.display = shouldShowCompleted ? 'block' : 'none';
        }

        if (abandonedSection) {
            const shouldShowAbandoned = abandonedVisible > 0 && (filter === 'all' || filter === 'abandoned');
            abandonedSection.style.display = shouldShowAbandoned ? 'block' : 'none';
        }

        if (unreadSection) {
            const shouldShowUnread = unreadVisible > 0 && (filter === 'all' || filter === 'unread');
            unreadSection.style.display = shouldShowUnread ? 'block' : 'none';
        }

        updateEmptyState(readingVisible + completedVisible + abandonedVisible + unreadVisible);
    }

    function updateEmptyState(visibleCount: number): void {
        const dynamicEmptyState = document.getElementById('dynamicEmptyState');

        if (visibleCount === 0) {
            // Show the dynamic empty state (for search/filter results)
            dynamicEmptyState?.classList.remove('hidden');
        } else {
            // Hide the dynamic empty state
            dynamicEmptyState?.classList.add('hidden');
        }
    }

    // Mobile search UI elements
    const mobileSearchButton = document.getElementById('mobileSearchButton');
    const mobileSearchContainer = document.getElementById('mobileSearchContainer');
    const mobileSearchClose = document.getElementById('mobileSearchClose');
    const mobileTitle = document.getElementById('mobileTitle');
    const mobileFilterControls = document.getElementById('mobileFilterControls');

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Focus search on "/" key
        if (e.key === '/' && !e.ctrlKey && !e.metaKey && !e.altKey) {
            e.preventDefault();
            if (window.innerWidth < 640) {
                mobileSearchButton?.click();
            } else if (searchInput) {
                searchInput.focus();
            }
        }

        // Clear search on Escape
        if (e.key === 'Escape') {
            if (document.activeElement === searchInput && searchInput) {
                searchInput.value = '';
                searchInput.dispatchEvent(new Event('input'));
                searchInput.blur();
            }
            if (!mobileSearchContainer?.classList.contains('hidden')) {
                mobileSearchClose?.click();
            }
        }

        // Filter shortcuts (Alt + number)
        if (e.altKey) {
            switch (e.key) {
                case '1':
                    e.preventDefault();
                    document.querySelector<HTMLElement>('[data-filter="all"]')?.click();
                    break;
                case '2':
                    e.preventDefault();
                    document.querySelector<HTMLElement>('[data-filter="reading"]')?.click();
                    break;
                case '3':
                    e.preventDefault();
                    document.querySelector<HTMLElement>('[data-filter="completed"]')?.click();
                    break;
                case '4':
                    e.preventDefault();
                    document.querySelector<HTMLElement>('[data-filter="abandoned"]')?.click();
                    break;
            }
        }
    });

    // Initialize progress bar animations
    const progressBars = document.querySelectorAll<HTMLElement>('.book-progress-bar');
    progressBars.forEach(bar => {
        const width = bar.style.width;
        bar.style.width = '0%';
        setTimeout(() => {
            bar.style.transition = 'width 1s ease-out';
            bar.style.width = width;
        }, 100);
    });

    // Unified dropdown filter logic
    const filterDropdownButton = document.getElementById('filterDropdownButton');
    const filterDropdownMenu = document.getElementById('filterDropdownMenu');
    const selectedFilterLabel = document.getElementById('selectedFilterLabel');

    filterDropdownButton?.addEventListener('click', () => {
        filterDropdownMenu?.classList.toggle('hidden');
    });

    filterDropdownMenu?.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        if (target.matches('button[data-filter]')) {
            const filterText = target.textContent;
            const filterType = target.dataset.filter || 'all';
            if (selectedFilterLabel && filterText) selectedFilterLabel.textContent = filterText;

            // Update filter icon color (gray when all, primary when filtered)
            const filterIcon = document.getElementById('filterIcon');
            if (filterIcon) {
                if (filterType === 'all') {
                    filterIcon.classList.remove('text-primary-500');
                    filterIcon.classList.add('text-gray-600', 'dark:text-gray-300');
                } else {
                    filterIcon.classList.remove('text-gray-600', 'dark:text-gray-300');
                    filterIcon.classList.add('text-primary-500');
                }
            }

            // Update aria-label and title based on filter type
            const filterAriaMap: Record<string, string> = {
                'all': 'filter.all-aria',
                'reading': 'filter.reading-aria',
                'completed': 'filter.completed-aria',
                'abandoned': 'filter.on-hold-aria',
                'unread': 'filter.unread-aria'
            };
            const ariaKey = filterAriaMap[filterType] || 'filter.all-aria';
            const ariaLabel = translation.get(ariaKey);
            if (filterDropdownButton) {
                filterDropdownButton.title = ariaLabel;
                filterDropdownButton.setAttribute('aria-label', ariaLabel);
            }

            filterDropdownMenu.classList.add('hidden');
        }
    });



    // Mobile search inline toggle logic
    mobileSearchButton?.addEventListener('click', () => {
        // Hide title, search button, and filter controls
        mobileTitle?.classList.add('hidden');
        mobileSearchButton?.classList.add('hidden');
        mobileFilterControls?.classList.add('hidden');

        // Show search container and close button
        mobileSearchContainer?.classList.remove('hidden');
        mobileSearchClose?.classList.remove('hidden');

        // Focus the search input
        setTimeout(() => mobileSearchInput?.focus(), 50);
    });

    mobileSearchClose?.addEventListener('click', () => {
        // Show title, search button, and filter controls
        mobileTitle?.classList.remove('hidden');
        mobileSearchButton?.classList.remove('hidden');
        mobileFilterControls?.classList.remove('hidden');

        // Hide search container and close button
        mobileSearchContainer?.classList.add('hidden');
        mobileSearchClose?.classList.add('hidden');

        // Clear search inputs
        if (mobileSearchInput) {
            mobileSearchInput.value = '';
        }
        if (searchInput) {
            searchInput.value = '';
            searchInput.dispatchEvent(new Event('input'));
        }
    });
});
