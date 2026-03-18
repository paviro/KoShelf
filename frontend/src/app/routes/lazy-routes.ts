import { lazy, type ComponentType, type LazyExoticComponent } from 'react';

import {
    matchRoute,
    type LibraryCollectionRoute,
    type RouteId,
} from './route-registry';

type PreloadableComponent<Props> = LazyExoticComponent<ComponentType<Props>> & {
    preload: () => Promise<void>;
};

function lazyWithPreload<Props>(
    importer: () => Promise<{ default: ComponentType<Props> }>,
): PreloadableComponent<Props> {
    const LazyComponent = lazy(importer) as PreloadableComponent<Props>;
    LazyComponent.preload = async () => {
        await importer();
    };
    return LazyComponent;
}

const importStatisticsRoute = async () => {
    const module =
        await import('../../features/statistics/routes/StatisticsRoute');
    return { default: module.StatisticsRoute };
};

const importCalendarRoute = async () => {
    const module = await import('../../features/calendar/routes/CalendarRoute');
    return { default: module.CalendarRoute };
};

const importSettingsRoute = async () => {
    const module = await import('../../features/settings/routes/SettingsRoute');
    return { default: module.SettingsRoute };
};

const importLoginRoute = async () => {
    const module = await import('../../features/auth/routes/LoginRoute');
    return { default: module.LoginRoute };
};

const importLibraryListRoute = async () => {
    const module =
        await import('../../features/library/routes/LibraryListRoute');
    return { default: module.LibraryListRoute };
};

const importLibraryDetailRoute = async () => {
    const module =
        await import('../../features/library/routes/LibraryDetailRoute');
    return { default: module.LibraryDetailRoute };
};

const importRecapRoute = async () => {
    const module = await import('../../features/recap/routes/RecapRoute');
    return { default: module.RecapRoute };
};

export const StatisticsRoute = lazyWithPreload(importStatisticsRoute);
export const CalendarRoute = lazyWithPreload(importCalendarRoute);
export const SettingsRoute = lazyWithPreload(importSettingsRoute);
export const LoginRoute = lazyWithPreload(importLoginRoute);
export const LibraryListRoute = lazyWithPreload<{
    collection: LibraryCollectionRoute;
}>(importLibraryListRoute);
export const LibraryDetailRoute = lazyWithPreload<{
    collection: LibraryCollectionRoute;
}>(importLibraryDetailRoute);
export const RecapRoute = lazyWithPreload(importRecapRoute);

const preloadedRoutePromises = new Map<RouteId, Promise<void>>();
const PRELOADERS_BY_ROUTE: Record<RouteId, Array<() => Promise<void>>> = {
    root: [],
    login: [LoginRoute.preload],
    statistics: [StatisticsRoute.preload],
    calendar: [CalendarRoute.preload],
    settings: [SettingsRoute.preload],
    'books-list': [LibraryListRoute.preload],
    'books-detail': [LibraryDetailRoute.preload],
    'comics-list': [LibraryListRoute.preload],
    'comics-detail': [LibraryDetailRoute.preload],
    recap: [RecapRoute.preload],
};

export function preloadRoute(routeId: RouteId): Promise<void> {
    const preloaders = PRELOADERS_BY_ROUTE[routeId];
    if (preloaders.length === 0) {
        return Promise.resolve();
    }

    const existingPromise = preloadedRoutePromises.get(routeId);
    if (existingPromise) {
        return existingPromise;
    }

    const promise = Promise.all(preloaders.map((preload) => preload()))
        .then(() => undefined)
        .catch((error: unknown) => {
            preloadedRoutePromises.delete(routeId);
            throw error;
        });

    preloadedRoutePromises.set(routeId, promise);
    return promise;
}

export function routePathnameFromHref(href: string): string | null {
    let url: URL;
    try {
        url = new URL(href, window.location.href);
    } catch {
        return null;
    }

    if (url.origin !== window.location.origin) {
        return null;
    }

    if (url.hash.length > 0) {
        if (!url.hash.startsWith('#/')) {
            return null;
        }

        const hashPath = url.hash.slice(1);
        const queryStart = hashPath.indexOf('?');
        return queryStart >= 0 ? hashPath.slice(0, queryStart) : hashPath;
    }

    return url.pathname;
}

export function matchRouteByHref(href: string) {
    const pathname = routePathnameFromHref(href);
    if (!pathname) {
        return null;
    }

    return matchRoute(pathname);
}
