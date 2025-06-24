// Book Details Page JavaScript
// Handles collapsible sections and other book details interactions

document.addEventListener('DOMContentLoaded', function() {
    // Initialize section toggles
    initializeSectionToggles();

    function initializeSectionToggles() {
        // Define toggle configurations for different sections
        const toggleConfigs = [
            {
                buttonId: 'toggleBookOverview',
                containerId: 'bookOverviewContainer',
                chevronId: 'bookOverviewChevron',
                showText: 'Show',
                hideText: 'Hide',
                defaultVisible: true  // Book overview shown by default
            },
            {
                buttonId: 'toggleReviewNote',
                containerId: 'reviewNoteContainer',
                chevronId: 'reviewNoteChevron',
                showText: 'Show',
                hideText: 'Hide',
                defaultVisible: true  // Review shown by default
            },
            {
                buttonId: 'toggleHighlights',
                containerId: 'highlightsContainer',
                chevronId: 'highlightsChevron',
                showText: 'Show',
                hideText: 'Hide',
                defaultVisible: true  // Highlights shown by default
            },
            {
                buttonId: 'toggleBookmarks',
                containerId: 'bookmarksContainer',
                chevronId: 'bookmarksChevron',
                showText: 'Show',
                hideText: 'Hide',
                defaultVisible: true  // Bookmarks shown by default
            },
            {
                buttonId: 'toggleReadingStats',
                containerId: 'readingStatsContainer',
                chevronId: 'readingStatsChevron',
                showText: 'Show Details',
                hideText: 'Hide Details',
                defaultVisible: false  // Stats hidden by default
            },
            {
                buttonId: 'toggleAdditionalInfo',
                containerId: 'additionalInfoContainer',
                chevronId: 'additionalInfoChevron',
                showText: 'Show Details',
                hideText: 'Hide Details',
                defaultVisible: false  // Additional info hidden by default
            }
        ];

        // Initialize each toggle
        toggleConfigs.forEach(config => {
            initializeToggle(config);
        });
    }

    function initializeToggle(config) {
        const toggleButton = document.getElementById(config.buttonId);
        const container = document.getElementById(config.containerId);
        const chevron = document.getElementById(config.chevronId);
        
        if (toggleButton && container && chevron) {
            const buttonTextElement = toggleButton.querySelector('span');
            
            if (buttonTextElement) {
                // Set initial state based on config
                setInitialState(config, container, chevron, buttonTextElement);
                
                // Add click event listener
                toggleButton.addEventListener('click', function() {
                    const isHidden = container.classList.contains('hidden');
                    
                    if (isHidden) {
                        // Show the section
                        container.classList.remove('hidden');
                        chevron.style.transform = 'rotate(180deg)';
                        buttonTextElement.textContent = config.hideText;
                    } else {
                        // Hide the section
                        container.classList.add('hidden');
                        chevron.style.transform = 'rotate(0deg)';
                        buttonTextElement.textContent = config.showText;
                    }
                });
            }
        }
    }

    function setInitialState(config, container, chevron, buttonTextElement) {
        if (config.defaultVisible) {
            // Show the section initially
            container.classList.remove('hidden');
            chevron.style.transform = 'rotate(180deg)';
            buttonTextElement.textContent = config.hideText;
        } else {
            // Hide the section initially
            container.classList.add('hidden');
            chevron.style.transform = 'rotate(0deg)';
            buttonTextElement.textContent = config.showText;
        }
    }
}); 