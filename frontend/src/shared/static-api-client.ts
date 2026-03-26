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
    ExportDayMetrics,
    ExportMonthMetrics,
    ExportPageActivityData,
    ExportReadingPeriods,
    ExportSite,
    LibraryDetailData,
    LibraryListData,
    LibraryListItem,
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

// ── Date helpers ────────────────────────────────────────────────────────

function mondayOfWeek(dateStr: string): string {
    const date = new Date(dateStr + 'T12:00:00');
    const day = date.getDay();
    const diff = day === 0 ? -6 : 1 - day;
    date.setDate(date.getDate() + diff);
    return date.toISOString().substring(0, 10);
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

        // Full-range: return the pre-computed summary.
        if (!from && !to) {
            return this.fetchCached<ReadingSummaryData>(
                `/data/reading/summary/${selectedScope}.json`,
            );
        }

        return this.computeRangedSummary(selectedScope, from, to);
    }

    async getReadingMetrics(
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ): Promise<ReadingMetricsData> {
        const selectedScope = normalizeScope(scope);
        return this.assembleMetrics(selectedScope, metric, groupBy, from, to);
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

    // ── Ranged summary computation ─────────────────────────────────────

    private async computeRangedSummary(
        scope: ScopeValue,
        from?: string,
        to?: string,
    ): Promise<ReadingSummaryData> {
        const periods = await this.fetchCached<ExportReadingPeriods>(
            `/data/reading/periods/${scope}.json`,
        );
        const relevantMonths = periods.reading_data.month.periods
            .map((p) => p.key)
            .filter((mk) => {
                if (from && mk < from.substring(0, 7)) return false;
                if (to && mk > to.substring(0, 7)) return false;
                return true;
            });

        let totalTime = 0;
        let totalPages = 0;
        let totalSessions = 0;
        let totalCompletions = 0;
        let maxTimeInDay = 0;
        let maxPagesInDay = 0;
        let longestSession = 0;

        for (const mk of relevantMonths) {
            let monthData: ExportMonthMetrics;
            try {
                monthData = await this.fetchCached<ExportMonthMetrics>(
                    `/data/reading/metrics/${mk}/${scope}.json`,
                );
            } catch {
                continue;
            }

            for (const [dayKey, d] of Object.entries(monthData.days)) {
                if (from && dayKey < from) continue;
                if (to && dayKey > to) continue;

                totalTime += d.reading_time_sec;
                totalPages += d.pages_read;
                totalSessions += d.sessions;
                totalCompletions += d.completions;
                if (d.reading_time_sec > maxTimeInDay)
                    maxTimeInDay = d.reading_time_sec;
                if (d.pages_read > maxPagesInDay) maxPagesInDay = d.pages_read;
                if (d.longest_session_duration_sec > longestSession)
                    longestSession = d.longest_session_duration_sec;
            }
        }

        // Grab heatmap_config from the full-range summary (global, not range-dependent).
        const fullSummary = await this.fetchCached<ReadingSummaryData>(
            `/data/reading/summary/${scope}.json`,
        );

        return {
            range: {
                from: from ?? '',
                to: to ?? '',
                tz: 'UTC',
            },
            overview: {
                reading_time_sec: totalTime,
                pages_read: totalPages,
                sessions: totalSessions,
                completions: totalCompletions,
                items_completed: 0,
                longest_reading_time_in_day_sec: maxTimeInDay,
                most_pages_in_day: maxPagesInDay,
                longest_session_duration_sec: longestSession || null,
                average_session_duration_sec:
                    totalSessions > 0
                        ? Math.round(totalTime / totalSessions)
                        : null,
            },
            streaks: {
                current: { days: 0 },
                longest: { days: 0 },
            },
            heatmap_config: fullSummary.heatmap_config,
        };
    }

    // ── Static metrics assembly ─────────────────────────────────────────

    private async assembleMetrics(
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ): Promise<ReadingMetricsData> {
        // Parse comma-separated metrics.
        const metricNames = metric
            .split(',')
            .map((m) => m.trim())
            .filter(Boolean);

        // Determine which month files to load from the periods index.
        const periods = await this.fetchCached<ExportReadingPeriods>(
            `/data/reading/periods/${scope}.json`,
        );
        const monthKeys = periods.reading_data.month.periods.map((p) => p.key);

        // Filter to months that overlap with the requested date range.
        const relevantMonths = monthKeys.filter((mk) => {
            if (from && mk < from.substring(0, 7)) return false;
            if (to && mk > to.substring(0, 7)) return false;
            return true;
        });

        // Load month files and extract metric data.
        const allPoints: Array<{
            key: string;
            values: Record<string, number>;
        }> = [];

        for (const mk of relevantMonths) {
            let monthData: ExportMonthMetrics;
            try {
                monthData = await this.fetchCached<ExportMonthMetrics>(
                    `/data/reading/metrics/${mk}/${scope}.json`,
                );
            } catch {
                continue;
            }

            for (const [dayKey, dayMetrics] of Object.entries(monthData.days)) {
                if (from && dayKey < from) continue;
                if (to && dayKey > to) continue;

                const values: Record<string, number> = {};
                for (const m of metricNames) {
                    values[m] =
                        (dayMetrics[m as keyof ExportDayMetrics] as number) ??
                        0;
                }
                allPoints.push({ key: dayKey, values });
            }
        }

        // Return day-level points directly.
        if (groupBy === 'day') {
            return {
                metrics: metricNames,
                group_by: groupBy,
                scope,
                points: allPoints.map((p) => ({ key: p.key, ...p.values })),
            };
        }

        // Aggregate into buckets (total, week, month, or year).
        const grouped = new Map<string, Record<string, number>>();
        for (const point of allPoints) {
            let groupKey: string;
            if (groupBy === 'total') {
                groupKey = 'total';
            } else if (groupBy === 'month') {
                groupKey = point.key.substring(0, 7);
            } else if (groupBy === 'year') {
                groupKey = point.key.substring(0, 4);
            } else {
                groupKey = mondayOfWeek(point.key);
            }
            const existing = grouped.get(groupKey);
            if (existing) {
                for (const m of metricNames) {
                    existing[m] = (existing[m] ?? 0) + (point.values[m] ?? 0);
                }
            } else {
                grouped.set(groupKey, { ...point.values });
            }
        }

        const points = Array.from(grouped.entries())
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([key, values]) => ({ key, ...values }));

        return { metrics: metricNames, group_by: groupBy, scope, points };
    }
}
