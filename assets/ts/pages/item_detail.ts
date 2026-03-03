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
        const shouldOpen = shareDropdownMenu?.classList.contains('hidden') ?? false;
        shareDropdownMenu?.classList.toggle('hidden');
        shareDropdownButton.setAttribute('aria-expanded', String(shouldOpen));
    });
});
