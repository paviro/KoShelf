import { useQueryClient } from '@tanstack/react-query';
import { useEffect, type ReactNode } from 'react';

import {
    libraryDetailCacheKey,
    prefetchLibraryDetailQuery,
    prefetchLibraryListQuery,
} from '../../features/library/hooks/useLibraryQueries';
import { prefetchStatisticsIndexQuery } from '../../features/statistics/hooks/useStatisticsQueries';
import { readStoredStatisticsViewState } from '../../features/statistics/model/statistics-model';
import { prefetchCalendarMonthsIfAvailable } from '../../features/calendar/hooks/useCalendarQueries';
import {
    loadInitialCalendarViewState,
    monthKey as toMonthKey,
    normalizeToMonthStart,
    shiftMonthKey,
} from '../../features/calendar/model/calendar-model';
import { prefetchRecapIndexQuery } from '../../features/recap/hooks/useRecapQueries';
import { readStoredRecapScope } from '../../features/recap/model/recap-model';
import { getPrefetchOnIntentPreference } from '../../shared/lib/network/prefetch-preference';
import { shouldPrefetchOnCurrentConnection } from '../../shared/lib/network/prefetch-guards';
import { matchRouteByHref, preloadRoute } from '../routes/lazy-routes';
import { RouteHeaderProvider } from './route-header';
import { ShellMobileNav } from './ShellMobileNav';
import { ShellSidebar } from './ShellSidebar';
import type { NavItem } from './shell-nav';

type AppShellProps = {
    navItems: NavItem[];
    currentPath: string;
    siteTitle: string;
    generatedAt?: string;
    version?: string;
    children: ReactNode;
};

export function AppShell({
    navItems,
    currentPath,
    siteTitle,
    generatedAt,
    version,
    children,
}: AppShellProps) {
    const queryClient = useQueryClient();

    useEffect(() => {
        const inFlightDetailPrefetches = new Set<string>();

        const preloadDataFromMatchedRoute = (
            matchedRoute: NonNullable<ReturnType<typeof matchRouteByHref>>,
        ) => {
            const { routeId } = matchedRoute;
            if (!routeId) {
                return;
            }

            if (routeId === 'statistics') {
                const statisticsScope = readStoredStatisticsViewState().scope;
                void prefetchStatisticsIndexQuery(queryClient, statisticsScope);
                return;
            }

            if (routeId === 'calendar') {
                const calendarViewState = loadInitialCalendarViewState();
                const calendarMonthKey =
                    calendarViewState.monthKey ??
                    toMonthKey(normalizeToMonthStart(new Date()));
                const calendarNeighborMonthKeys = [
                    shiftMonthKey(calendarMonthKey, -1),
                    calendarMonthKey,
                    shiftMonthKey(calendarMonthKey, 1),
                ];

                void prefetchCalendarMonthsIfAvailable(
                    queryClient,
                    calendarNeighborMonthKeys,
                );
                return;
            }

            if (routeId === 'recap') {
                const recapScope = readStoredRecapScope();
                void prefetchRecapIndexQuery(queryClient, recapScope);
                return;
            }

            if (routeId === 'books-list' || routeId === 'comics-list') {
                const collection =
                    routeId === 'comics-list' ? 'comics' : 'books';
                void prefetchLibraryListQuery(queryClient, collection);
                return;
            }

            if (routeId === 'books-detail' || routeId === 'comics-detail') {
                const itemId = matchedRoute.params.id;
                if (!itemId) {
                    return;
                }

                const collection =
                    routeId === 'comics-detail' ? 'comics' : 'books';
                const detailKey = libraryDetailCacheKey(collection, itemId);
                if (inFlightDetailPrefetches.has(detailKey)) {
                    return;
                }

                inFlightDetailPrefetches.add(detailKey);
                void prefetchLibraryDetailQuery(
                    queryClient,
                    collection,
                    itemId,
                ).finally(() => {
                    inFlightDetailPrefetches.delete(detailKey);
                });
            }
        };

        const resolveAnchor = (target: EventTarget | null) => {
            if (!(target instanceof Element)) {
                return null;
            }

            const anchor = target.closest('a[href]');
            if (!(anchor instanceof HTMLAnchorElement)) {
                return null;
            }

            if (anchor.target === '_blank' || anchor.hasAttribute('download')) {
                return null;
            }

            return anchor;
        };

        const preloadFromAnchor = (anchor: HTMLAnchorElement) => {
            const matchedRoute = matchRouteByHref(anchor.href);
            if (!matchedRoute?.routeId) {
                return;
            }

            if (!getPrefetchOnIntentPreference()) {
                return;
            }

            if (!shouldPrefetchOnCurrentConnection()) {
                return;
            }

            void preloadRoute(matchedRoute.routeId);
            preloadDataFromMatchedRoute(matchedRoute);
        };

        const handlePointerOver = (event: PointerEvent) => {
            const anchor = resolveAnchor(event.target);
            if (!anchor) {
                return;
            }

            const relatedAnchor =
                event.relatedTarget instanceof Element
                    ? event.relatedTarget.closest('a[href]')
                    : null;
            if (relatedAnchor === anchor) {
                return;
            }

            preloadFromAnchor(anchor);
        };
        const handleFocusIn = (event: FocusEvent) => {
            const anchor = resolveAnchor(event.target);
            if (!anchor) {
                return;
            }
            preloadFromAnchor(anchor);
        };
        const handleTouchStart = (event: TouchEvent) => {
            const anchor = resolveAnchor(event.target);
            if (!anchor) {
                return;
            }
            preloadFromAnchor(anchor);
        };

        document.addEventListener('pointerover', handlePointerOver, true);
        document.addEventListener('focusin', handleFocusIn, true);
        document.addEventListener('touchstart', handleTouchStart, true);
        return () => {
            document.removeEventListener(
                'pointerover',
                handlePointerOver,
                true,
            );
            document.removeEventListener('focusin', handleFocusIn, true);
            document.removeEventListener('touchstart', handleTouchStart, true);
        };
    }, [queryClient]);

    return (
        <div className="min-h-full bg-gray-100 dark:bg-dark-925 text-gray-900 dark:text-white font-sans">
            <ShellSidebar
                navItems={navItems}
                currentPath={currentPath}
                siteTitle={siteTitle}
                generatedAt={generatedAt}
                version={version}
            />
            <ShellMobileNav navItems={navItems} currentPath={currentPath} />

            <RouteHeaderProvider
                currentPath={currentPath}
                siteTitle={siteTitle}
            >
                <div className="min-h-full lg:ml-64">{children}</div>
            </RouteHeaderProvider>
        </div>
    );
}
