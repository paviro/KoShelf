import {
    buildRoutePath,
    isMainRouteId,
    matchRoute,
    type MainRouteId,
} from '../../../app/routes/route-registry';

export type DetailReturnState = {
    detailReturnRouteId?: MainRouteId;
    detailReturnSearch?: string;
};

function normalizeInternalPath(path: unknown): string | null {
    if (
        typeof path !== 'string' ||
        !path.startsWith('/') ||
        path.startsWith('//')
    ) {
        return null;
    }

    try {
        const normalized = new URL(path, window.location.origin);
        if (normalized.origin !== window.location.origin) {
            return null;
        }

        return `${normalized.pathname}${normalized.search}`;
    } catch {
        return null;
    }
}

function normalizeSearch(search: unknown): string {
    if (typeof search !== 'string') {
        return '';
    }

    const trimmed = search.trim();
    if (!trimmed) {
        return '';
    }

    if (trimmed.startsWith('?')) {
        return trimmed;
    }

    return `?${trimmed}`;
}

export function createDetailReturnState(
    pathname: string,
    search = '',
): DetailReturnState {
    const matched = matchRoute(pathname);
    if (!isMainRouteId(matched.routeId)) {
        return {};
    }

    const normalizedPath = buildRoutePath(matched.routeId);
    const normalized = normalizeInternalPath(
        `${normalizedPath}${normalizeSearch(search)}`,
    );
    if (!normalized) {
        return {};
    }

    const normalizedUrl = new URL(normalized, window.location.origin);
    return {
        detailReturnRouteId: matched.routeId,
        detailReturnSearch: normalizeSearch(normalizedUrl.search),
    };
}

export function resolveDetailReturnPath(state: unknown): string | null {
    if (!state || typeof state !== 'object') {
        return null;
    }

    const candidate = state as DetailReturnState;
    if (
        candidate.detailReturnRouteId &&
        isMainRouteId(candidate.detailReturnRouteId)
    ) {
        const path = `${buildRoutePath(candidate.detailReturnRouteId)}${normalizeSearch(candidate.detailReturnSearch)}`;
        return normalizeInternalPath(path);
    }

    return null;
}
