import type {
    ApiClient,
    CompletionsParams,
    ScopeValue,
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from './api-client';
import { normalizeScope } from './api-client';
import { fetchJson } from './api-fetch';
import type {
    PageActivityData,
    ExportPageActivityData,
    ExportReadingPeriods,
    ExportSite,
    LibraryDetailData,
    LibraryListData,
    LibraryListItem,
    MetricPoint,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
    SessionInfo,
    SiteData,
} from './contracts';

function authUnavailableError(): Error {
    return new Error('Authentication is unavailable in static mode.');
}

function writeUnavailableError(): Error {
    return new Error('Write operations are unavailable in static mode.');
}

// ── Helpers ─────────────────────────────────────────────────────────────

function filterPointsByRange(
    points: MetricPoint[],
    from?: string,
    to?: string,
): MetricPoint[] {
    if (!from && !to) return points;
    return points.filter(
        (p) => (!from || p.key >= from) && (!to || p.key <= to),
    );
}

function monthKeysForRange(from?: string, to?: string): string[] {
    if (!from || !to) return [];
    const keys: string[] = [];
    let current = from.substring(0, 7);
    const end = to.substring(0, 7);
    while (current <= end) {
        keys.push(current);
        const [y, m] = current.split('-').map(Number);
        const next =
            m === 12 ? `${y + 1}-01` : `${y}-${String(m + 1).padStart(2, '0')}`;
        current = next;
    }
    return keys;
}

// ── StaticApiClient ─────────────────────────────────────────────────────

export class StaticApiClient implements ApiClient {
    private cache = new Map<string, unknown>();

    private async fetchCached<T>(path: string): Promise<T> {
        const cached = this.cache.get(path);
        if (cached !== undefined) {
            return cached as T;
        }
        const data = await fetchJson(path);
        this.cache.set(path, data);
        return data as T;
    }

    async getSite(): Promise<SiteData> {
        const exported = (await fetchJson('/data/site.json')) as ExportSite;
        return {
            title: exported.name,
            language: exported.default_language,
            capabilities: exported.capabilities,
            version: exported.version,
            generated_at: exported.generated_at,
        };
    }

    async login(password: string): Promise<void> {
        void password;
        throw authUnavailableError();
    }

    async getSessions(): Promise<SessionInfo[]> {
        throw authUnavailableError();
    }

    async revokeSession(sessionId: string): Promise<void> {
        void sessionId;
        throw authUnavailableError();
    }

    async changePassword(
        currentPassword: string,
        newPassword: string,
    ): Promise<void> {
        void currentPassword;
        void newPassword;
        throw authUnavailableError();
    }

    async logout(): Promise<void> {
        throw authUnavailableError();
    }

    async getItems(scope?: ScopeValue): Promise<LibraryListData> {
        const selectedScope = normalizeScope(scope);
        const file = selectedScope === 'all' ? 'index' : selectedScope;
        const payload = await fetchJson(`/data/items/${file}.json`);

        const items = Array.isArray(payload)
            ? (payload as LibraryListItem[])
            : ((payload as LibraryListData).items ?? []);

        return { items };
    }

    async getItem(id: string): Promise<LibraryDetailData> {
        return (await fetchJson(`/data/items/${id}.json`)) as LibraryDetailData;
    }

    async getReadingSummary(
        scope: ScopeValue,
        from?: string,
        to?: string,
    ): Promise<ReadingSummaryData> {
        const selectedScope = normalizeScope(scope);

        if (!from || !to) {
            return this.fetchCached<ReadingSummaryData>(
                `/data/reading/summary/${selectedScope}.json`,
            );
        }

        // Year-aligned range: YYYY-01-01 to YYYY-12-31
        if (
            from.endsWith('-01-01') &&
            to.endsWith('-12-31') &&
            from.substring(0, 4) === to.substring(0, 4)
        ) {
            const year = from.substring(0, 4);
            return this.fetchCached<ReadingSummaryData>(
                `/data/reading/summary/year/${year}/${selectedScope}.json`,
            );
        }

        // Week-aligned range: key is the Monday date (from).
        return this.fetchCached<ReadingSummaryData>(
            `/data/reading/summary/week/${from}/${selectedScope}.json`,
        );
    }

    async getReadingMetrics(
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ): Promise<ReadingMetricsData> {
        const selectedScope = normalizeScope(scope);
        const metrics = metric
            .split(',')
            .map((m) => m.trim())
            .filter(Boolean);

        if (groupBy === 'total') {
            const data = await this.fetchCached<ReadingMetricsData>(
                `/data/reading/metrics/total/${selectedScope}.json`,
            );
            return { ...data, metrics };
        }

        if (groupBy === 'year') {
            const data = await this.fetchCached<ReadingMetricsData>(
                `/data/reading/metrics/year/${selectedScope}.json`,
            );
            return {
                ...data,
                metrics,
                points: filterPointsByRange(data.points, from, to),
            };
        }

        if ((groupBy === 'month' || groupBy === 'week') && from) {
            const year = from.substring(0, 4);
            const data = await this.fetchCached<ReadingMetricsData>(
                `/data/reading/metrics/${groupBy}/${year}/${selectedScope}.json`,
            );
            return { ...data, metrics };
        }

        // Day-level: load relevant month files and concatenate.
        const months = monthKeysForRange(from, to);
        const allPoints: MetricPoint[] = [];
        for (const mk of months) {
            try {
                const data = await this.fetchCached<ReadingMetricsData>(
                    `/data/reading/metrics/day/${mk}/${selectedScope}.json`,
                );
                allPoints.push(...data.points);
            } catch {
                continue;
            }
        }
        return {
            metrics,
            group_by: groupBy,
            scope: selectedScope,
            points: filterPointsByRange(allPoints, from, to),
        };
    }

    async getAvailablePeriods(
        source: string,
        groupBy: string,
        scope: ScopeValue,
    ): Promise<ReadingAvailablePeriodsData> {
        const selectedScope = normalizeScope(scope);
        const exported = await this.fetchCached<ExportReadingPeriods>(
            `/data/reading/periods/${selectedScope}.json`,
        );

        const sourceData = exported[source as keyof ExportReadingPeriods];
        return sourceData[
            groupBy as keyof typeof sourceData
        ] as ReadingAvailablePeriodsData;
    }

    async getReadingCalendar(
        month: string,
        scope: ScopeValue,
    ): Promise<ReadingCalendarData> {
        const selectedScope = normalizeScope(scope);
        return (await fetchJson(
            `/data/reading/calendar/${month}/${selectedScope}.json`,
        )) as ReadingCalendarData;
    }

    async getReadingCompletions(
        scope: ScopeValue,
        params: CompletionsParams,
    ): Promise<ReadingCompletionsData> {
        const selectedScope = normalizeScope(scope);
        const year = params.year ?? new Date().getFullYear();
        return (await fetchJson(
            `/data/reading/completions/${year}/${selectedScope}.json`,
        )) as ReadingCompletionsData;
    }

    async getItemPageActivity(
        id: string,
        completion?: string,
    ): Promise<PageActivityData> {
        let exported: ExportPageActivityData;
        try {
            exported = (await fetchJson(
                `/data/items/page-activity/${id}.json`,
            )) as ExportPageActivityData;
        } catch {
            return {
                total_pages: 0,
                pages: [],
                annotations: [],
            };
        }

        if (completion !== undefined && completion !== 'all') {
            const pages = exported.by_completion[completion];
            return {
                ...exported,
                pages: pages ?? [],
            };
        }

        return exported;
    }

    getItemDownloadHref(id: string): string {
        return `/data/items/${id}.json`;
    }

    getItemFileHref(id: string, format?: string | null): string | null {
        if (!format) return null;
        return `/assets/files/${encodeURIComponent(id)}.${encodeURIComponent(format)}`;
    }

    async updateItem(id: string, payload: UpdateItemPayload): Promise<void> {
        void id;
        void payload;
        throw writeUnavailableError();
    }

    async updateAnnotation(
        itemId: string,
        annotationId: string,
        payload: UpdateAnnotationPayload,
    ): Promise<void> {
        void itemId;
        void annotationId;
        void payload;
        throw writeUnavailableError();
    }

    async deleteAnnotation(
        itemId: string,
        annotationId: string,
    ): Promise<void> {
        void itemId;
        void annotationId;
        throw writeUnavailableError();
    }

    clearCache(): void {
        this.cache.clear();
    }
}
