export type ServerMode = 'internal' | 'external';
export type ScopeValue = 'all' | 'books' | 'comics';

type LibraryCollection = 'books' | 'comics';
type ContractRoute = {
    apiPath: string;
    dataPath: string;
};

const SERVER_MODE_STORAGE_KEY = 'koshelf_server_mode';

declare global {
    interface Window {
        __KOSHELF_SERVER_MODE?: ServerMode;
    }
}

function parseStoredServerMode(raw: string | null): ServerMode | null {
    if (!raw) return null;

    try {
        const parsed = JSON.parse(raw) as unknown;
        if (parsed === 'internal' || parsed === 'external') {
            return parsed;
        }
    } catch {
        // Ignore malformed storage values.
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
        stored = parseStoredServerMode(localStorage.getItem(SERVER_MODE_STORAGE_KEY));
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

function normalizeScope(scope: ScopeValue | undefined): ScopeValue {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }

    return 'all';
}

function withScope(path: string, scope: ScopeValue): string {
    const separator = path.includes('?') ? '&' : '?';
    return `${path}${separator}scope=${scope}`;
}

function projectScopedPayload(payload: unknown, scope: ScopeValue): unknown {
    if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
        return payload;
    }

    const root = payload as Record<string, unknown>;
    const scopesValue = root.scopes;

    if (!scopesValue || typeof scopesValue !== 'object' || Array.isArray(scopesValue)) {
        return payload;
    }

    const scopes = scopesValue as Record<string, unknown>;
    const selected = scopes[scope];

    const projected: Record<string, unknown> = { ...root };
    delete projected.scopes;

    if (selected && typeof selected === 'object' && !Array.isArray(selected)) {
        Object.assign(projected, selected as Record<string, unknown>);
    }

    return projected;
}

async function fetchJson(url: string): Promise<unknown> {
    const response = await fetch(url, {
        method: 'GET',
        headers: { Accept: 'application/json' },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch ${url} (${response.status})`);
    }

    return response.json();
}

function route(apiPath: string, dataPath: string): ContractRoute {
    return { apiPath, dataPath };
}

async function request<T>(target: ContractRoute): Promise<T> {
    const url = isServeMode() ? target.apiPath : target.dataPath;
    return (await fetchJson(url)) as T;
}

async function requestScoped<T>(target: ContractRoute, scope: ScopeValue | undefined): Promise<T> {
    const selectedScope = normalizeScope(scope);

    if (isServeMode()) {
        return (await fetchJson(withScope(target.apiPath, selectedScope))) as T;
    }

    const payload = await fetchJson(target.dataPath);
    return projectScopedPayload(payload, selectedScope) as T;
}

function parseLibraryHref(href: string): { collection: LibraryCollection; id: string } | null {
    const url = new URL(href, window.location.origin);
    const match = url.pathname.match(/^\/(books|comics)\/([^/]+)\/?/);
    if (!match) {
        return null;
    }

    const collection = match[1];
    const id = match[2];

    if ((collection !== 'books' && collection !== 'comics') || !id) {
        return null;
    }

    return { collection, id };
}

export const api = {
    site: {
        async get<T>(): Promise<T> {
            return request<T>(route('/api/site', '/data/site.json'));
        },
    },

    locales: {
        async get<T>(): Promise<T> {
            return request<T>(route('/api/locales', '/data/locales.json'));
        },
    },

    books: {
        async list<T>(): Promise<T> {
            return request<T>(route('/api/books', '/data/books.json'));
        },

        async get<T>(id: string): Promise<T> {
            return request<T>(route(`/api/books/${id}`, `/data/books/${id}.json`));
        },
    },

    comics: {
        async list<T>(): Promise<T> {
            return request<T>(route('/api/comics', '/data/comics.json'));
        },

        async get<T>(id: string): Promise<T> {
            return request<T>(route(`/api/comics/${id}`, `/data/comics/${id}.json`));
        },
    },

    statistics: {
        async get<T>(scope?: ScopeValue): Promise<T> {
            return requestScoped<T>(route('/api/statistics', '/data/statistics/index.json'), scope);
        },

        weeks: {
            async get<T>(weekKey: string, scope?: ScopeValue): Promise<T> {
                return requestScoped<T>(
                    route(
                        `/api/statistics/weeks/${weekKey}`,
                        `/data/statistics/weeks/${weekKey}.json`,
                    ),
                    scope,
                );
            },
        },

        years: {
            async get<T>(year: number, scope?: ScopeValue): Promise<T> {
                return requestScoped<T>(
                    route(`/api/statistics/years/${year}`, `/data/statistics/years/${year}.json`),
                    scope,
                );
            },
        },
    },

    calendar: {
        months: {
            async list<T>(): Promise<T> {
                return request<T>(route('/api/calendar/months', '/data/calendar/months.json'));
            },

            async get<T>(monthKey: string): Promise<T> {
                return request<T>(
                    route(
                        `/api/calendar/months/${monthKey}`,
                        `/data/calendar/months/${monthKey}.json`,
                    ),
                );
            },
        },
    },

    recap: {
        async get<T>(scope?: ScopeValue): Promise<T> {
            return requestScoped<T>(route('/api/recap', '/data/recap/index.json'), scope);
        },

        years: {
            async get<T>(year: number, scope?: ScopeValue): Promise<T> {
                return requestScoped<T>(
                    route(`/api/recap/years/${year}`, `/data/recap/years/${year}.json`),
                    scope,
                );
            },
        },
    },

    library: {
        async getByHref<T>(href: string): Promise<T | null> {
            const parsed = parseLibraryHref(href);
            if (!parsed) {
                return null;
            }

            if (parsed.collection === 'books') {
                return api.books.get<T>(parsed.id);
            }

            return api.comics.get<T>(parsed.id);
        },
    },
};
