import type {
    ApiResponse,
    ExportDayMetricsByScope,
    ExportMonthMetrics,
    ExportPeriodsByScope,
    ExportReadingPeriods,
    ExportReadingSummary,
    ExportSite,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
} from './contracts';

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
    return isServeMode() ? `/api/items/${id}` : `/data/items/${id}.json`;
}

function normalizeScope(scope: ScopeValue | undefined): ScopeValue {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
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

function appendParams(
    base: string,
    params: Record<string, string | undefined>,
): string {
    const separator = base.includes('?') ? '&' : '?';
    const parts: string[] = [];
    for (const [key, value] of Object.entries(params)) {
        if (value !== undefined) {
            parts.push(`${key}=${encodeURIComponent(value)}`);
        }
    }
    return parts.length > 0 ? `${base}${separator}${parts.join('&')}` : base;
}

// ── Static-mode JSON cache ──────────────────────────────────────────────

const staticCache = new Map<string, unknown>();

async function fetchStaticJson<T>(path: string): Promise<T> {
    const cached = staticCache.get(path);
    if (cached !== undefined) {
        return cached as T;
    }
    const data = await fetchJson(path);
    staticCache.set(path, data);
    return data as T;
}

export function clearStaticCache(): void {
    staticCache.clear();
}

// ── Items helpers ───────────────────────────────────────────────────────

type LibraryListPayload = {
    items: Array<{
        content_type?: string;
    }>;
};

function filterItemsPayload(payload: unknown, scope: ScopeValue): unknown {
    if (scope === 'all') {
        return payload;
    }

    const expected = scope === 'books' ? 'book' : 'comic';

    // Static mode: items/index.json is a raw array of items.
    if (Array.isArray(payload)) {
        return {
            items: (payload as Array<{ content_type?: string }>).filter(
                (item) => item.content_type === expected,
            ),
        };
    }

    if (!payload || typeof payload !== 'object') {
        return payload;
    }

    const typed = payload as LibraryListPayload;
    if (!Array.isArray(typed.items)) {
        return payload;
    }

    return {
        ...(payload as Record<string, unknown>),
        items: typed.items.filter((item) => item.content_type === expected),
    };
}

async function requestItemsList<T>(scope: ScopeValue | undefined): Promise<T> {
    const selectedScope = normalizeScope(scope);

    if (isServeMode()) {
        return (await fetchJson(
            appendParams('/api/items', { scope: selectedScope }),
        )) as T;
    }

    const payload = await fetchJson('/data/items/index.json');

    // Static export writes a raw array; wrap it to match serve-mode shape.
    if (Array.isArray(payload)) {
        return filterItemsPayload(payload, selectedScope) as T;
    }

    return filterItemsPayload(payload, selectedScope) as T;
}

// ── Static-mode reading helpers ─────────────────────────────────────────

function pickScope<T>(
    byScope: { all: T; books: T; comics: T },
    scope: ScopeValue,
): T {
    return byScope[scope];
}

// ── Public API ──────────────────────────────────────────────────────────

export const api = {
    site: {
        async get<T>(): Promise<T> {
            if (isServeMode()) {
                return (await fetchJson('/api/site')) as T;
            }

            const exported = (await fetchJson('/data/site.json')) as ExportSite;
            return {
                meta: { version: exported.version, generated_at: '' },
                title: exported.name,
                language: exported.default_language,
                capabilities: exported.capabilities,
            } as T;
        },
    },

    items: {
        async list<T>(scope?: ScopeValue): Promise<T> {
            return requestItemsList<T>(scope);
        },

        async get<T>(id: string): Promise<T> {
            if (isServeMode()) {
                return (await fetchJson(`/api/items/${id}?include=all`)) as T;
            }
            return (await fetchJson(`/data/items/${id}.json`)) as T;
        },
    },

    reading: {
        async summary(
            scope: ScopeValue,
            from?: string,
            to?: string,
        ): Promise<ReadingSummaryData> {
            const selectedScope = normalizeScope(scope);

            if (isServeMode()) {
                const url = appendParams('/api/reading/summary', {
                    scope: selectedScope,
                    from,
                    to,
                });
                const response = (await fetchJson(
                    url,
                )) as ApiResponse<ReadingSummaryData>;
                return response.data;
            }

            const exported = await fetchStaticJson<ExportReadingSummary>(
                '/data/reading/summary.json',
            );
            return pickScope(exported, selectedScope);
        },

        async availablePeriods(
            source: string,
            groupBy: string,
            scope: ScopeValue,
        ): Promise<ReadingAvailablePeriodsData> {
            const selectedScope = normalizeScope(scope);

            if (isServeMode()) {
                const url = appendParams('/api/reading/available-periods', {
                    source,
                    group_by: groupBy,
                    scope: selectedScope,
                });
                const response = (await fetchJson(
                    url,
                )) as ApiResponse<ReadingAvailablePeriodsData>;
                return response.data;
            }

            const exported = await fetchStaticJson<ExportReadingPeriods>(
                '/data/reading/periods.json',
            );

            const sourceData = exported[source as keyof ExportReadingPeriods];
            const groupData = sourceData[
                groupBy as keyof typeof sourceData
            ] as ExportPeriodsByScope;
            return pickScope(groupData, selectedScope);
        },

        async metrics(
            scope: ScopeValue,
            metric: string,
            groupBy: string,
            from?: string,
            to?: string,
        ): Promise<ReadingMetricsData> {
            const selectedScope = normalizeScope(scope);

            if (isServeMode()) {
                const url = appendParams('/api/reading/metrics', {
                    scope: selectedScope,
                    metric,
                    group_by: groupBy,
                    from,
                    to,
                });
                const response = (await fetchJson(
                    url,
                )) as ApiResponse<ReadingMetricsData>;
                return response.data;
            }

            // Static mode: load per-month metric files and assemble.
            return loadStaticMetrics(selectedScope, metric, groupBy, from, to);
        },

        async calendar(
            month: string,
            scope: ScopeValue,
        ): Promise<ReadingCalendarData> {
            const selectedScope = normalizeScope(scope);

            if (isServeMode()) {
                const url = appendParams('/api/reading/calendar', {
                    month,
                    scope: selectedScope,
                });
                const response = (await fetchJson(
                    url,
                )) as ApiResponse<ReadingCalendarData>;
                return response.data;
            }

            return (await fetchJson(
                `/data/reading/calendar/${month}.json`,
            )) as ReadingCalendarData;
        },

        async completions(
            scope: ScopeValue,
            params: {
                year?: number;
                from?: string;
                to?: string;
                groupBy?: string;
                include?: string;
            },
        ): Promise<ReadingCompletionsData> {
            const selectedScope = normalizeScope(scope);

            if (isServeMode()) {
                const url = appendParams('/api/reading/completions', {
                    scope: selectedScope,
                    year:
                        params.year !== undefined
                            ? String(params.year)
                            : undefined,
                    from: params.from,
                    to: params.to,
                    group_by: params.groupBy,
                    include: params.include,
                });
                const response = (await fetchJson(
                    url,
                )) as ApiResponse<ReadingCompletionsData>;
                return response.data;
            }

            // Static export stores per-year files with scope=all, group_by=month, all includes.
            // Client-side filtering by scope is needed.
            const year = params.year ?? new Date().getFullYear();
            const data = (await fetchJson(
                `/data/reading/completions/${year}.json`,
            )) as ReadingCompletionsData;
            return filterCompletionsByScope(data, selectedScope);
        },
    },
};

// ── Static metrics assembly ─────────────────────────────────────────────

async function loadStaticMetrics(
    scope: ScopeValue,
    metric: string,
    groupBy: string,
    from?: string,
    to?: string,
): Promise<ReadingMetricsData> {
    // Determine which month files to load from the periods index.
    const periods = await fetchStaticJson<ExportReadingPeriods>(
        '/data/reading/periods.json',
    );
    const monthPeriods = pickScope(periods.reading_data.month, scope);
    const monthKeys = monthPeriods.periods.map((p) => p.key);

    // Filter to months that overlap with the requested date range.
    const relevantMonths = monthKeys.filter((mk) => {
        if (from && mk < from.substring(0, 7)) return false;
        if (to && mk > to.substring(0, 7)) return false;
        return true;
    });

    // Load month files and extract metric+scope data.
    const allPoints: Array<{ key: string; value: number }> = [];

    for (const mk of relevantMonths) {
        let monthData: ExportMonthMetrics;
        try {
            monthData = await fetchStaticJson<ExportMonthMetrics>(
                `/data/reading/metrics/${mk}.json`,
            );
        } catch {
            continue;
        }

        for (const [dayKey, byScope] of Object.entries(monthData.days)) {
            if (from && dayKey < from) continue;
            if (to && dayKey > to) continue;

            const scopeMetrics: ExportDayMetricsByScope[keyof ExportDayMetricsByScope] =
                byScope[scope];
            const value =
                scopeMetrics[metric as keyof typeof scopeMetrics] ?? 0;
            allPoints.push({ key: dayKey, value });
        }
    }

    // Handle group_by aggregation for week/month.
    if (groupBy === 'day') {
        return {
            metric,
            group_by: groupBy,
            scope,
            points: allPoints,
        };
    }

    const grouped = new Map<string, number>();
    for (const point of allPoints) {
        const groupKey =
            groupBy === 'month'
                ? point.key.substring(0, 7)
                : mondayOfWeek(point.key);
        grouped.set(groupKey, (grouped.get(groupKey) ?? 0) + point.value);
    }

    const points = Array.from(grouped.entries())
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, value]) => ({ key, value }));

    return { metric, group_by: groupBy, scope, points };
}

function mondayOfWeek(dateStr: string): string {
    const date = new Date(dateStr + 'T12:00:00');
    const day = date.getDay();
    const diff = day === 0 ? -6 : 1 - day;
    date.setDate(date.getDate() + diff);
    return date.toISOString().substring(0, 10);
}

// ── Static completions scope filter ─────────────────────────────────────

function filterCompletionsByScope(
    data: ReadingCompletionsData,
    scope: ScopeValue,
): ReadingCompletionsData {
    if (scope === 'all') return data;

    const expected = scope === 'books' ? 'book' : 'comic';
    const matchesScope = (item: { content_type?: string | null }) =>
        item.content_type === expected;

    const filteredGroups = data.groups
        ?.map((group) => {
            const items = group.items.filter(matchesScope);
            return {
                ...group,
                items,
                items_finished: items.length,
                reading_time_sec: items.reduce(
                    (sum, i) => sum + i.reading_time_sec,
                    0,
                ),
            };
        })
        .filter((g) => g.items.length > 0);

    const filteredItems = data.items?.filter(matchesScope);

    return {
        ...data,
        groups: filteredGroups,
        items: filteredItems,
    };
}
