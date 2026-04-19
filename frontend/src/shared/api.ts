import type { ApiClient } from './api-client';
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

type ServerMode = 'internal' | 'external';

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

function getServerMode(): ServerMode {
    if (
        window.__KOSHELF_SERVER_MODE === 'internal' ||
        window.__KOSHELF_SERVER_MODE === 'external'
    ) {
        return window.__KOSHELF_SERVER_MODE;
    }

    let stored: ServerMode | null;
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

export const api = new Proxy<ApiClient>({} as ApiClient, {
    get(_target, prop: string | symbol) {
        const client = getClient();
        const value = client[prop as keyof ApiClient];
        return typeof value === 'function'
            ? (value as (...args: unknown[]) => unknown).bind(client)
            : value;
    },
});
