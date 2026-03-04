import type { IconType } from 'react-icons';
import {
    LuBookMarked,
    LuBookOpen,
    LuCalendarDays,
    LuChartNoAxesColumn,
    LuClock3,
} from 'react-icons/lu';

import type { SiteResponse } from '../../shared/contracts';
import { translation } from '../../shared/i18n';
import {
    matchRoute,
    resolveMainRouteId,
    type MainRouteId,
} from '../routes/route-registry';

export type NavItem = {
    id?: string;
    routeId: MainRouteId;
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

export function buildNavItems(site: SiteResponse | undefined): NavItem[] {
    if (!site) return [];

    const items: NavItem[] = [];
    const capabilities = site.capabilities;

    if (capabilities.has_books) {
        items.push({
            routeId: 'books-list',
            label: translation.get('books'),
            href: '/books',
            icon: ICONS.books,
        });
    }

    if (capabilities.has_comics) {
        items.push({
            routeId: 'comics-list',
            label: translation.get('comics'),
            href: '/comics',
            icon: ICONS.comics,
        });
    }

    if (capabilities.has_statistics) {
        items.push({
            id: 'nav-statistics',
            routeId: 'statistics',
            label: translation.get('statistics'),
            href: '/statistics',
            icon: ICONS.statistics,
        });

        items.push({
            routeId: 'calendar',
            label: translation.get('calendar'),
            href: '/calendar',
            icon: ICONS.calendar,
        });
    }

    if (capabilities.has_recap) {
        items.push({
            id: 'nav-recap',
            routeId: 'recap',
            label: translation.get('recap'),
            href: '/recap',
            icon: ICONS.recap,
        });
    }

    return items;
}

export function isActivePath(
    currentPath: string,
    routeId: MainRouteId,
): boolean {
    const currentRoute = matchRoute(currentPath);
    if (!currentRoute.routeId) {
        return false;
    }

    const currentMainRoute = resolveMainRouteId(currentRoute.routeId);
    return currentMainRoute === routeId;
}
