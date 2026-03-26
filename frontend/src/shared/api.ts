import type {
    ApiClient,
    CompletionsParams,
    ScopeValue,
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from './api-client';
import { HttpApiClient } from './http-api-client';
import { StaticApiClient } from './static-api-client';

export type {
    ApiClient,
    CompletionsParams,
    ScopeValue,
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from './api-client';
export { ApiHttpError, isApiHttpError } from './api-fetch';

// ── Server mode detection ───────────────────────────────────────────────

export type ServerMode = 'internal' | 'external';

declare global {
    interface Window {
        __KOSHELF_SERVER_MODE?: ServerMode;
    }
}

const SERVER_MODE_STORAGE_KEY = 'koshelf_server_mode';

function parseStoredServerMode(raw: string | null): ServerMode | null {
    if (!raw) return null;

    try {
        const parsed = JSON.parse(raw) as unknown;
        if (parsed === 'internal' || parsed === 'external') {
            return parsed;
        }
    } catch {
        // Ignore malformed values.
    }

    return null;
}

export function getServerMode(): ServerMode {
    if (
        window.__KOSHELF_SERVER_MODE === 'internal' ||
        window.__KOSHELF_SERVER_MODE === 'external'
    ) {
        return window.__KOSHELF_SERVER_MODE;
    }

    let stored: ServerMode | null = null;
    try {
        stored = parseStoredServerMode(
            localStorage.getItem(SERVER_MODE_STORAGE_KEY),
        );
    } catch {
        stored = null;
    }

    if (stored) {
        return stored;
    }

    return 'external';
}

export function isServeMode(): boolean {
    return getServerMode() === 'internal';
}

// ── Lazy client creation ────────────────────────────────────────────────
//
// The client is created on first use rather than at module load time.
// This ensures `window.__KOSHELF_SERVER_MODE` has been set by the
// entry point before the mode check runs.

let _client: ApiClient | null = null;

function getClient(): ApiClient {
    if (!_client) {
        _client = isServeMode() ? new HttpApiClient() : new StaticApiClient();
    }
    return _client;
}

export const api: ApiClient = {
    getSite: () => getClient().getSite(),
    login: (password: string) => getClient().login(password),
    getSessions: () => getClient().getSessions(),
    revokeSession: (sessionId: string) => getClient().revokeSession(sessionId),
    changePassword: (currentPassword: string, newPassword: string) =>
        getClient().changePassword(currentPassword, newPassword),
    logout: () => getClient().logout(),
    getItems: (scope?: ScopeValue) => getClient().getItems(scope),
    getItem: (id: string) => getClient().getItem(id),
    getItemPageActivity: (id: string, completion?: string) =>
        getClient().getItemPageActivity(id, completion),
    getReadingSummary: (scope: ScopeValue, from?: string, to?: string) =>
        getClient().getReadingSummary(scope, from, to),
    getReadingMetrics: (
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ) => getClient().getReadingMetrics(scope, metric, groupBy, from, to),
    getAvailablePeriods: (source: string, groupBy: string, scope: ScopeValue) =>
        getClient().getAvailablePeriods(source, groupBy, scope),
    getReadingCalendar: (month: string, scope: ScopeValue) =>
        getClient().getReadingCalendar(month, scope),
    getReadingCompletions: (scope: ScopeValue, params: CompletionsParams) =>
        getClient().getReadingCompletions(scope, params),
    getItemDownloadHref: (id: string) => getClient().getItemDownloadHref(id),
    getItemFileHref: (id: string, format?: string | null) =>
        getClient().getItemFileHref(id, format),
    updateItem: (id: string, payload: UpdateItemPayload) =>
        getClient().updateItem(id, payload),
    updateAnnotation: (
        itemId: string,
        annotationId: string,
        payload: UpdateAnnotationPayload,
    ) => getClient().updateAnnotation(itemId, annotationId, payload),
    deleteAnnotation: (itemId: string, annotationId: string) =>
        getClient().deleteAnnotation(itemId, annotationId),
    clearCache: () => getClient().clearCache(),
};
