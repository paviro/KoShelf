/**
 * Filter Restore Module
 *
 * Stores the current filter scope (all/books/comics) when visiting recap/stats pages
 * and intercepts navigation to restore the last-selected scope.
 *
 * Progressive enhancement: without JS, links work as normal.
 */

import { StorageManager } from './storage-manager.js';

type Scope = 'all' | 'books' | 'comics';

/**
 * Detect current page type and scope from URL path
 */
function detectPageContext(): { type: 'recap' | 'statistics' | null; scope: Scope; year?: string } {
    const path = window.location.pathname;

    // Match /statistics/, /statistics/books/, /statistics/comics/
    const statsMatch = path.match(/^\/statistics\/(books|comics)?/);
    if (statsMatch) {
        const scope = (statsMatch[1] as Scope) || 'all';
        return { type: 'statistics', scope };
    }

    // Match /recap/{year}/, /recap/{year}/books/, /recap/{year}/comics/
    const recapMatch = path.match(/^\/recap\/(\d{4})\/(books|comics)?/);
    if (recapMatch) {
        const year = recapMatch[1];
        const scope = (recapMatch[2] as Scope) || 'all';
        return { type: 'recap', scope, year };
    }

    return { type: null, scope: 'all' };
}

/**
 * Save the current scope to localStorage based on page type
 */
function saveCurrentScope(): void {
    const context = detectPageContext();

    if (context.type === 'statistics') {
        StorageManager.set(StorageManager.KEYS.STATS_FILTER, context.scope);
    } else if (context.type === 'recap') {
        StorageManager.set(StorageManager.KEYS.RECAP_FILTER, context.scope);
    }
}

/**
 * Build the scoped URL for a given base href and scope
 */
function buildScopedUrl(href: string, scope: Scope): string {
    if (scope === 'all') {
        return href; // No modification needed for 'all'
    }

    // Ensure href ends with /
    const base = href.endsWith('/') ? href : href + '/';
    return base + scope + '/';
}

/**
 * Intercept navigation clicks on sidebar and tab bar
 */
function setupNavigationInterception(): void {
    // Target both sidebar and bottom navbar links by ID
    const statsLinks = document.querySelectorAll<HTMLAnchorElement>('#nav-statistics');
    const recapLinks = document.querySelectorAll<HTMLAnchorElement>('#nav-recap');

    // Handle Statistics links
    statsLinks.forEach((link) => {
        link.addEventListener('click', (e) => {
            const storedScope = StorageManager.get<Scope>(StorageManager.KEYS.STATS_FILTER);
            if (storedScope && storedScope !== 'all') {
                e.preventDefault();
                const newUrl = buildScopedUrl(link.href, storedScope);
                window.location.href = newUrl;
            }
            // If no scope stored or scope is 'all', let the default navigation happen
        });
    });

    // Handle Recap links
    recapLinks.forEach((link) => {
        link.addEventListener('click', (e) => {
            const storedScope = StorageManager.get<Scope>(StorageManager.KEYS.RECAP_FILTER);
            if (storedScope && storedScope !== 'all') {
                e.preventDefault();
                const newUrl = buildScopedUrl(link.href, storedScope);
                window.location.href = newUrl;
            }
            // If no scope stored or scope is 'all', let the default navigation happen
        });
    });
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    // Save current scope on page load
    saveCurrentScope();

    // Set up navigation interception
    setupNavigationInterception();
});
