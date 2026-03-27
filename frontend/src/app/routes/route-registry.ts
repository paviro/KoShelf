import { generatePath, matchPath } from 'react-router';

export type RouteId =
    | 'root'
    | 'login'
    | 'statistics'
    | 'calendar'
    | 'settings'
    | 'books-list'
    | 'books-detail'
    | 'books-read'
    | 'comics-list'
    | 'comics-detail'
    | 'comics-read'
    | 'recap';

export type MainRouteId =
    | 'statistics'
    | 'calendar'
    | 'settings'
    | 'books-list'
    | 'comics-list'
    | 'recap';
export type DetailRouteId = 'books-detail' | 'comics-detail';
export type ReaderRouteId = 'books-read' | 'comics-read';
export type LibraryCollectionRoute = 'books' | 'comics';
type LibraryContentTypeRoute = 'book' | 'comic';

type RouteDefinition = {
    id: RouteId;
    path: string;
    mainRouteId: MainRouteId | null;
};

const ROUTE_DEFINITIONS: readonly RouteDefinition[] = [
    { id: 'root', path: '/', mainRouteId: null },
    { id: 'login', path: '/login', mainRouteId: null },
    { id: 'statistics', path: '/statistics', mainRouteId: 'statistics' },
    { id: 'calendar', path: '/calendar', mainRouteId: 'calendar' },
    { id: 'settings', path: '/settings', mainRouteId: 'settings' },
    { id: 'books-list', path: '/books', mainRouteId: 'books-list' },
    { id: 'books-detail', path: '/books/:id', mainRouteId: 'books-list' },
    { id: 'books-read', path: '/books/:id/read', mainRouteId: 'books-list' },
    { id: 'comics-list', path: '/comics', mainRouteId: 'comics-list' },
    { id: 'comics-detail', path: '/comics/:id', mainRouteId: 'comics-list' },
    { id: 'comics-read', path: '/comics/:id/read', mainRouteId: 'comics-list' },
    { id: 'recap', path: '/recap', mainRouteId: 'recap' },
] as const;

const ROUTE_DEFINITION_BY_ID: Record<RouteId, RouteDefinition> =
    Object.fromEntries(
        ROUTE_DEFINITIONS.map((definition) => [definition.id, definition]),
    ) as Record<RouteId, RouteDefinition>;

const MAIN_ROUTE_IDS = [
    'books-list',
    'comics-list',
    'statistics',
    'calendar',
    'settings',
    'recap',
] as const;

export type ScrollableRouteId = MainRouteId | DetailRouteId;

const READER_ROUTE_IDS: ReadonlySet<string> = new Set<string>([
    'books-read',
    'comics-read',
]);

const SCROLLABLE_ROUTE_IDS: ReadonlySet<string> = new Set<string>([
    ...MAIN_ROUTE_IDS,
    'books-detail',
    'comics-detail',
]);

export function isScrollableRouteId(
    routeId: RouteId | null,
): routeId is ScrollableRouteId {
    return routeId !== null && SCROLLABLE_ROUTE_IDS.has(routeId);
}

type RouteMatch = {
    routeId: RouteId | null;
    mainRouteId: MainRouteId | null;
    params: Record<string, string>;
    normalizedPathname: string;
};

function normalizePathname(pathname: string): string {
    if (pathname === '/') {
        return '/';
    }

    return pathname.replace(/\/+$/, '') || '/';
}

export function buildRoutePath(
    routeId: RouteId,
    params?: {
        id?: string;
    },
): string {
    const route = ROUTE_DEFINITION_BY_ID[routeId];
    if (route.path.includes(':id')) {
        const id = params?.id;
        if (!id) {
            throw new Error(
                `Missing required route param "id" for route ${routeId}`,
            );
        }
        return generatePath(route.path, { id });
    }

    return route.path;
}

export function routePathPattern(routeId: RouteId): string {
    return ROUTE_DEFINITION_BY_ID[routeId].path;
}

export function matchRoute(pathname: string): RouteMatch {
    const normalizedPathname = normalizePathname(pathname);

    for (const route of ROUTE_DEFINITIONS) {
        const matched = matchPath(
            {
                path: route.path,
                end: true,
            },
            normalizedPathname,
        );
        if (!matched) {
            continue;
        }

        const params: Record<string, string> = {};
        Object.entries(matched.params).forEach(([key, value]) => {
            if (typeof value === 'string' && value.length > 0) {
                params[key] = value;
            }
        });

        return {
            routeId: route.id,
            mainRouteId: route.mainRouteId,
            params,
            normalizedPathname,
        };
    }

    return {
        routeId: null,
        mainRouteId: null,
        params: {},
        normalizedPathname,
    };
}

export function isReaderRouteId(
    routeId: RouteId | null,
): routeId is ReaderRouteId {
    return routeId !== null && READER_ROUTE_IDS.has(routeId);
}

export function isMainRouteId(routeId: RouteId | null): routeId is MainRouteId {
    return routeId !== null && MAIN_ROUTE_IDS.includes(routeId as MainRouteId);
}

export function resolveMainRouteId(
    routeId: RouteId | null,
): MainRouteId | null {
    if (!routeId) {
        return null;
    }

    return ROUTE_DEFINITION_BY_ID[routeId].mainRouteId;
}

export function detailRouteIdForCollection(
    collection: LibraryCollectionRoute,
): DetailRouteId {
    return collection === 'comics' ? 'comics-detail' : 'books-detail';
}

export function readerRouteIdForCollection(
    collection: LibraryCollectionRoute,
): ReaderRouteId {
    return collection === 'comics' ? 'comics-read' : 'books-read';
}

export function detailRouteIdForContentType(
    contentType: LibraryContentTypeRoute,
): DetailRouteId {
    return contentType === 'comic' ? 'comics-detail' : 'books-detail';
}

export function listRouteIdForCollection(
    collection: LibraryCollectionRoute,
): MainRouteId {
    return collection === 'comics' ? 'comics-list' : 'books-list';
}
