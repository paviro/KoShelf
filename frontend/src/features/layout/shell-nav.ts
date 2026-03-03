import type { RecapIndexResponse, SiteResponse } from '../../shared/contracts';
import { translation } from '../../shared/i18n';

export type NavItem = {
    id?: string;
    label: string;
    href: string;
    iconSvg: string;
};

const ICONS = {
    books: 'M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253',
    comics: 'M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z',
    statistics:
        'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z',
    calendar:
        'M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
    recap: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
} as const;

export const BRAND_ICON_SVG = ICONS.books;

export function buildNavItems(
    site: SiteResponse | undefined,
    recapIndex: RecapIndexResponse | undefined,
): NavItem[] {
    if (!site) return [];

    const items: NavItem[] = [];
    const capabilities = site.capabilities;

    if (capabilities.has_books) {
        items.push({
            label: translation.get('books'),
            href: '/books',
            iconSvg: ICONS.books,
        });
    }

    if (capabilities.has_comics) {
        items.push({
            label: translation.get('comics'),
            href: '/comics',
            iconSvg: ICONS.comics,
        });
    }

    if (capabilities.has_statistics) {
        items.push({
            id: 'nav-statistics',
            label: translation.get('statistics'),
            href: '/statistics',
            iconSvg: ICONS.statistics,
        });

        items.push({
            label: translation.get('calendar'),
            href: '/calendar',
            iconSvg: ICONS.calendar,
        });
    }

    if (capabilities.has_recap) {
        const latestYear = recapIndex?.latest_year ?? recapIndex?.available_years?.[0];
        const recapHref = latestYear ? `/recap/${latestYear}` : '/recap';
        items.push({
            id: 'nav-recap',
            label: translation.get('recap'),
            href: recapHref,
            iconSvg: ICONS.recap,
        });
    }

    return items;
}

export function isActivePath(currentPath: string, href: string): boolean {
    if (href === '/statistics') {
        return currentPath.startsWith('/statistics');
    }
    if (href.startsWith('/recap')) {
        return currentPath.startsWith('/recap');
    }
    return currentPath === href;
}
