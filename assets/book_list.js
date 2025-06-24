// KoInsight - Reading Tracker Interface
import { LazyImageLoader } from './lazy-loading.js';

document.addEventListener('DOMContentLoaded', function() {
    const searchInput = document.getElementById('searchInput');
    const filterButtons = document.querySelectorAll('[data-filter]');
    const readingSection = document.querySelector('section:has(#readingBooksGrid)');
    const completedSection = document.querySelector('section:has(#completedBooksGrid)');
    const unreadSection = document.querySelector('section:has(#unreadBooksGrid)');
    const bookCards = document.querySelectorAll('.book-card');
    
    let currentFilter = 'all';
    
    // Initialize lazy loading
    const lazyLoader = new LazyImageLoader();
    lazyLoader.init();
    
    // Check for search query in URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const searchQuery = urlParams.get('search');
    
    if (searchQuery && searchInput) {
        // Set the search input value
        searchInput.value = searchQuery;
        
        // Remove the search parameter from URL without refreshing the page
        const url = new URL(window.location);
        url.searchParams.delete('search');
        window.history.replaceState({}, document.title, url.toString());
        
        // Trigger the search
        setTimeout(() => {
            filterBooks(searchQuery.toLowerCase().trim(), currentFilter);
        }, 100);
    }
    
    // Search functionality
    if (searchInput) {
        searchInput.addEventListener('input', function() {
            const searchTerm = this.value.toLowerCase().trim();
            filterBooks(searchTerm, currentFilter);
        });
    }
    
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
            if (searchInput) {
                searchInput.focus();
            }
        }
        
        // Clear search on Escape
        if (e.key === 'Escape' && searchInput && document.activeElement === searchInput) {
            searchInput.value = '';
            searchInput.dispatchEvent(new Event('input'));
            searchInput.blur();
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

    // Section toggle functionality
    const sectionToggles = [
        {
            buttonId: 'toggleReading',
            contentId: 'readingBooksGrid',
            chevronId: 'readingChevron'
        },
        {
            buttonId: 'toggleCompleted',
            contentId: 'completedBooksGrid',
            chevronId: 'completedChevron'
        },
        {
            buttonId: 'toggleUnread',
            contentId: 'unreadBooksGrid',
            chevronId: 'unreadChevron'
        }
    ];

    sectionToggles.forEach(config => {
        const button = document.getElementById(config.buttonId);
        const content = document.getElementById(config.contentId);
        const chevron = document.getElementById(config.chevronId);
        
        if (button && content && chevron) {
            button.addEventListener('click', () => {
                const isHidden = content.classList.contains('hidden');
                const buttonText = button.querySelector('span');
                
                if (isHidden) {
                    // Show content
                    content.classList.remove('hidden');
                    buttonText.textContent = 'Hide';
                    chevron.style.transform = 'rotate(0deg)';
                } else {
                    // Hide content
                    content.classList.add('hidden');
                    buttonText.textContent = 'Show';
                    chevron.style.transform = 'rotate(-90deg)';
                }
            });
        }
    });

}); 