// Book Details Page JavaScript
// Uses the SectionToggle module for handling collapsible sections

import { SectionToggle } from './section-toggle.js';

document.addEventListener('DOMContentLoaded', function() {
    // Initialize section toggles using the module
    const sectionToggle = new SectionToggle();
    
    // Share dropdown toggle logic
    const shareDropdownButton = document.getElementById('shareDropdownButton');
    const shareDropdownMenu = document.getElementById('shareDropdownMenu');

    shareDropdownButton?.addEventListener('click', () => {
        shareDropdownMenu?.classList.toggle('hidden');
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', (e) => {
        if (!shareDropdownButton?.contains(e.target) && !shareDropdownMenu?.contains(e.target)) {
            shareDropdownMenu?.classList.add('hidden');
        }
    });
}); 