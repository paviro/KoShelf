// KoInsight - Reading Tracker Interface
import { LazyImageLoader } from './lazy-loading.js';
import { SectionToggle } from './section-toggle.js';

document.addEventListener('DOMContentLoaded', function() {
    const searchInput = document.getElementById('searchInput');
    const mobileSearchInput = document.getElementById('mobileSearchInput');
    const filterButtons = document.querySelectorAll('[data-filter]');
    const readingSection = document.querySelector('section:has(#readingContainer)');
    const completedSection = document.querySelector('section:has(#completedContainer)');
    const unreadSection = document.querySelector('section:has(#unreadContainer)');
    const bookCards = document.querySelectorAll('.book-card');
    
    let currentFilter = 'all';
    
    // Initialize lazy loading
    const lazyLoader = new LazyImageLoader();
    lazyLoader.init();
    
    // Initialize section toggles
    const sectionToggle = new SectionToggle();
    
    // Make it globally available for debugging or external control
    window.bookListSections = sectionToggle;
    
    // Check for search query in URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const searchQuery = urlParams.get('search');
    
    if (searchQuery) {
        if (searchInput) searchInput.value = searchQuery;
        if (mobileSearchInput) mobileSearchInput.value = searchQuery;
        
        // Remove the search parameter from URL without refreshing the page
        const url = new URL(window.location);
        url.searchParams.delete('search');
        window.history.replaceState({}, document.title, url.toString());
        
        // Trigger the search
        setTimeout(() => {
            filterBooks(searchQuery.toLowerCase().trim(), currentFilter);
        }, 100);
    }
    
    let preSearchSectionState = null;
    let lastSearchTerm = '';
    
    // Unified search handler
    function handleSearchInput(value) {
        const searchTerm = value.toLowerCase().trim();
        // Save state only on first non-empty search
        if (searchTerm && !lastSearchTerm) {
            preSearchSectionState = {};
            sectionToggle.getSectionNames().forEach(name => {
                preSearchSectionState[name] = sectionToggle.isVisible(name);
            });
        }
        // Restore state when search is cleared
        if (!searchTerm && lastSearchTerm) {
            if (preSearchSectionState) {
                sectionToggle.getSectionNames().forEach(name => {
                    if (preSearchSectionState[name]) {
                        sectionToggle.show(name);
                    } else {
                        sectionToggle.hide(name);
                    }
                });
            }
            preSearchSectionState = null;
        }
        lastSearchTerm = searchTerm;
        filterBooks(searchTerm, currentFilter);
        // After filtering, expand/collapse sections based on visible books
        if (searchTerm) {
            // For each section, check if it has visible books
            ['reading', 'completed', 'unread'].forEach(name => {
                const container = document.getElementById(name + 'Container');
                if (!container) return;
                const hasVisible = Array.from(container.children).some(child => child.style.display !== 'none');
                if (hasVisible) {
                    sectionToggle.show(name);
                } else {
                    sectionToggle.hide(name);
                }
            });
        }
    }
    
    // Attach input listeners to both inputs (if they exist)
    [searchInput, mobileSearchInput].forEach(inp => {
        if (!inp) return;
        inp.addEventListener('input', function() {
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
        button.addEventListener('click', function() {
            // Update filter button states
            filterButtons.forEach(btn => btn.classList.remove('filter-button-active'));
            this.classList.add('filter-button-active');
            
            currentFilter = this.dataset.filter;
            const searchTerm = searchInput ? searchInput.value.toLowerCase().trim() : '';
            filterBooks(searchTerm, currentFilter);
        });
    });
    
    function filterBooks(searchTerm, filter) {
        let readingVisible = 0;
        let completedVisible = 0;
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
                
                // Load images for newly visible cards that haven't loaded yet
                lazyLoader.loadImageForCard(card);
                
                // Count visible books by status
                if (status === 'reading') readingVisible++;
                if (status === 'completed') completedVisible++;
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
        
        if (unreadSection) {
            const shouldShowUnread = unreadVisible > 0 && (filter === 'all' || filter === 'unread');
            unreadSection.style.display = shouldShowUnread ? 'block' : 'none';
        }
        
        updateEmptyState(readingVisible + completedVisible + unreadVisible);
    }
    
    function updateEmptyState(visibleCount) {
        const dynamicEmptyState = document.getElementById('dynamicEmptyState');
        
        if (visibleCount === 0) {
            // Show the dynamic empty state (for search/filter results)
            if (dynamicEmptyState) {
                dynamicEmptyState.classList.remove('hidden');
            }
        } else {
            // Hide the dynamic empty state
            if (dynamicEmptyState) {
                dynamicEmptyState.classList.add('hidden');
            }
        }
    }
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(e) {
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
            if (document.activeElement === searchInput) {
                searchInput.value = '';
                searchInput.dispatchEvent(new Event('input'));
                searchInput.blur();
            }
            if (!mobileSearchOverlay?.classList.contains('hidden')) {
                mobileSearchClose?.click();
            }
        }
        
        // Filter shortcuts (Alt + number)
        if (e.altKey) {
            switch(e.key) {
                case '1':
                    e.preventDefault();
                    document.querySelector('[data-filter="all"]')?.click();
                    break;
                case '2':
                    e.preventDefault();
                    document.querySelector('[data-filter="reading"]')?.click();
                    break;
                case '3':
                    e.preventDefault();
                    document.querySelector('[data-filter="completed"]')?.click();
                    break;
            }
        }
    });
    
    // Initialize progress bar animations
    const progressBars = document.querySelectorAll('.book-progress-bar');
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
    const selectedFilterLabelMobile = document.getElementById('selectedFilterLabelMobile');

    filterDropdownButton?.addEventListener('click', () => {
        filterDropdownMenu.classList.toggle('hidden');
    });

    filterDropdownMenu?.addEventListener('click', (e) => {
        if (e.target.matches('button[data-filter]')) {
            const filterText = e.target.textContent;
            if (selectedFilterLabel) selectedFilterLabel.textContent = filterText;
            if (selectedFilterLabelMobile) selectedFilterLabelMobile.textContent = filterText;
            filterDropdownMenu.classList.add('hidden');
        }
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', (e) => {
        if (!filterDropdownButton?.contains(e.target) && !filterDropdownMenu?.contains(e.target)) {
            filterDropdownMenu?.classList.add('hidden');
        }
    });

    // Mobile search UI elements
    const mobileSearchButton = document.getElementById('mobileSearchButton');
    const mobileSearchOverlay = document.getElementById('mobileSearchOverlay');
    const mobileSearchClose = document.getElementById('mobileSearchClose');

    // Mobile search overlay toggle logic
    mobileSearchButton?.addEventListener('click', () => {
        mobileSearchOverlay?.classList.remove('hidden');
        setTimeout(() => mobileSearchInput?.focus(), 50);
    });

    mobileSearchClose?.addEventListener('click', () => {
        mobileSearchOverlay?.classList.add('hidden');
        if (mobileSearchInput) {
            mobileSearchInput.value = '';
        }
        if (searchInput) {
            searchInput.value = '';
            searchInput.dispatchEvent(new Event('input'));
        }
    });
}); 