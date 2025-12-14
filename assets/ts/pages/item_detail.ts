// Book Details Page JavaScript
// Uses the SectionToggle module for handling collapsible sections

import { SectionToggle } from '../components/section-toggle.js';

document.addEventListener('DOMContentLoaded', () => {
    // Initialize section toggles using the module
    new SectionToggle();

    // Share dropdown toggle logic
    const shareDropdownButton = document.getElementById('shareDropdownButton');
    const shareDropdownMenu = document.getElementById('shareDropdownMenu');

    shareDropdownButton?.addEventListener('click', () => {
        shareDropdownMenu?.classList.toggle('hidden');
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', (e) => {
        const target = e.target as Node;
        if (!shareDropdownButton?.contains(target) && !shareDropdownMenu?.contains(target)) {
            shareDropdownMenu?.classList.add('hidden');
        }
    });
});
