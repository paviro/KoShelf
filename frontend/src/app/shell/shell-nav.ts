import type { IconType } from 'react-icons';
import {
    LuBookMarked,
    LuBookOpen,
    LuCalendarDays,
    LuChartNoAxesColumn,
    LuClock3,
} from 'react-icons/lu';

import type { RecapIndexResponse, SiteResponse } from '../../shared/contracts';
import { translation } from '../../shared/i18n';

export type NavItem = {
    id?: string;
    label: string;
    href: string;
    icon: IconType;
};

const ICONS = {
    books: LuBookOpen,
    comics: LuBookMarked,
    statistics: LuChartNoAxesColumn,
    calendar: LuCalendarDays,
    recap: LuClock3,
} as const;

export const BRAND_ICON = ICONS.books;

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
            icon: ICONS.books,
        });
    }

    if (capabilities.has_comics) {
        items.push({
            label: translation.get('comics'),
            href: '/comics',
            icon: ICONS.comics,
        });
    }

    if (capabilities.has_statistics) {
        items.push({
            id: 'nav-statistics',
            label: translation.get('statistics'),
            href: '/statistics',
            icon: ICONS.statistics,
        });

        items.push({
            label: translation.get('calendar'),
            href: '/calendar',
            icon: ICONS.calendar,
        });
    }

    if (capabilities.has_recap) {
        const latestYear = recapIndex?.latest_year ?? recapIndex?.available_years?.[0];
        const recapHref = latestYear ? `/recap/${latestYear}` : '/recap';
        items.push({
            id: 'nav-recap',
            label: translation.get('recap'),
            href: recapHref,
            icon: ICONS.recap,
        });
    }

    return items;
}

export function isActivePath(currentPath: string, href: string): boolean {
    if (href === '/statistics') {
        return currentPath.startsWith('/statistics');
    }
    if (href === '/books') {
        return currentPath.startsWith('/books');
    }
    if (href === '/comics') {
        return currentPath.startsWith('/comics');
    }
    if (href.startsWith('/recap')) {
        return currentPath.startsWith('/recap');
    }
    return currentPath === href;
}
