export type ServerMode = 'internal' | 'external';
export type ScopeValue = 'all' | 'books' | 'comics';

export class ApiHttpError extends Error {
    readonly status: number;
    readonly url: string;

    constructor(url: string, status: number) {
        super(`Failed to fetch ${url} (${status})`);
        this.name = 'ApiHttpError';
        this.status = status;
        this.url = url;
    }
}

export function isApiHttpError(error: unknown): error is ApiHttpError {
    return error instanceof ApiHttpError;
}

type ContractRoute = {
    apiPath: string;
    dataPath: string;
};

type ContentTypeRoute = {
    apiPath: string;
    dataPathFor: (scope: ScopeValue) => string;
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

export function itemDetailDownloadHref(id: string): string {
    return isServeMode() ? `/api/items/${id}` : `/data/items/by_id/${id}.json`;
}

function normalizeScope(scope: ScopeValue | undefined): ScopeValue {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}

function withContentType(path: string, scope: ScopeValue): string {
    const separator = path.includes('?') ? '&' : '?';
    return `${path}${separator}content_type=${scope}`;
}

async function fetchJson(url: string): Promise<unknown> {
    const response = await fetch(url, {
        method: 'GET',
        headers: { Accept: 'application/json' },
    });

    if (!response.ok) {
        throw new ApiHttpError(url, response.status);
    }

    return response.json();
}

function route(apiPath: string, dataPath: string): ContractRoute {
    return { apiPath, dataPath };
}

function routeByContentType(
    apiPath: string,
    dataPathFor: (scope: ScopeValue) => string,
): ContentTypeRoute {
    return { apiPath, dataPathFor };
}

async function request<T>(target: ContractRoute): Promise<T> {
    const url = isServeMode() ? target.apiPath : target.dataPath;
    return (await fetchJson(url)) as T;
}

async function requestByContentType<T>(
    target: ContentTypeRoute,
    scope: ScopeValue | undefined,
): Promise<T> {
    const selectedScope = normalizeScope(scope);

    if (isServeMode()) {
        return (await fetchJson(
            withContentType(target.apiPath, selectedScope),
        )) as T;
    }

    return (await fetchJson(target.dataPathFor(selectedScope))) as T;
}

type LibraryListPayload = {
    items: Array<{
        content_type?: string;
    }>;
};

function filterItemsPayload(payload: unknown, scope: ScopeValue): unknown {
    if (scope === 'all') {
        return payload;
    }

    if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
        return payload;
    }

    const typed = payload as LibraryListPayload;
    if (!Array.isArray(typed.items)) {
        return payload;
    }

    const expected = scope === 'books' ? 'book' : 'comic';

    return {
        ...(payload as Record<string, unknown>),
        items: typed.items.filter((item) => item.content_type === expected),
    };
}

async function requestItemsList<T>(scope: ScopeValue | undefined): Promise<T> {
    const selectedScope = normalizeScope(scope);

    if (isServeMode()) {
        return (await fetchJson(
            withContentType('/api/items', selectedScope),
        )) as T;
    }

    const payload = await fetchJson('/data/items/index.json');
    return filterItemsPayload(payload, selectedScope) as T;
}

export const api = {
    site: {
        async get<T>(): Promise<T> {
            return request<T>(route('/api/site', '/data/site.json'));
        },
    },

    items: {
        async list<T>(scope?: ScopeValue): Promise<T> {
            return requestItemsList<T>(scope);
        },

        async get<T>(id: string): Promise<T> {
            return request<T>(
                route(`/api/items/${id}`, `/data/items/by_id/${id}.json`),
            );
        },
    },

    activity: {
        weeks: {
            async get<T>(scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        '/api/activity/weeks',
                        (selectedScope) =>
                            `/data/activity/weeks/${selectedScope}/index.json`,
                    ),
                    scope,
                );
            },

            async byKey<T>(weekKey: string, scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        `/api/activity/weeks/${weekKey}`,
                        (selectedScope) =>
                            `/data/activity/weeks/${selectedScope}/by_key/${weekKey}.json`,
                    ),
                    scope,
                );
            },
        },

        years: {
            daily: {
                async get<T>(year: number, scope?: ScopeValue): Promise<T> {
                    return requestByContentType<T>(
                        routeByContentType(
                            `/api/activity/years/${year}/daily`,
                            (selectedScope) =>
                                `/data/activity/years/${selectedScope}/daily/${year}.json`,
                        ),
                        scope,
                    );
                },
            },
            summary: {
                async get<T>(year: number, scope?: ScopeValue): Promise<T> {
                    return requestByContentType<T>(
                        routeByContentType(
                            `/api/activity/years/${year}/summary`,
                            (selectedScope) =>
                                `/data/activity/years/${selectedScope}/summary/${year}.json`,
                        ),
                        scope,
                    );
                },
            },
        },

        months: {
            async list<T>(scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        '/api/activity/months',
                        (selectedScope) =>
                            `/data/activity/months/${selectedScope}/index.json`,
                    ),
                    scope,
                );
            },

            async get<T>(monthKey: string, scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        `/api/activity/months/${monthKey}`,
                        (selectedScope) =>
                            `/data/activity/months/${selectedScope}/by_key/${monthKey}.json`,
                    ),
                    scope,
                );
            },
        },
    },

    completions: {
        years: {
            async get<T>(scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        '/api/completions/years',
                        (selectedScope) =>
                            `/data/completions/years/${selectedScope}/index.json`,
                    ),
                    scope,
                );
            },

            async byKey<T>(year: number, scope?: ScopeValue): Promise<T> {
                return requestByContentType<T>(
                    routeByContentType(
                        `/api/completions/years/${year}`,
                        (selectedScope) =>
                            `/data/completions/years/${selectedScope}/by_key/${year}.json`,
                    ),
                    scope,
                );
            },
        },
    },
};
