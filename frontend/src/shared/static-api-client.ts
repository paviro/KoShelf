import type { ApiClient, CompletionsParams, ScopeValue } from './api-client';
import { normalizeScope } from './api-client';
import { fetchJson } from './api-fetch';
import type {
    ExportDayMetricsByScope,
    ExportMonthMetrics,
    ExportPeriodsByScope,
    ExportReadingPeriods,
    ExportReadingSummary,
    ExportSite,
    LibraryDetailData,
    LibraryListData,
    LibraryListItem,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
    SiteData,
} from './contracts';

function pickScope<T>(
    byScope: { all: T; books: T; comics: T },
    scope: ScopeValue,
): T {
    return byScope[scope];
}

// ── Items helpers ───────────────────────────────────────────────────────

function filterItemsByScope(
    items: LibraryListItem[],
    scope: ScopeValue,
): LibraryListItem[] {
    if (scope === 'all') return items;
    const expected = scope === 'books' ? 'book' : 'comic';
    return items.filter((item) => item.content_type === expected);
}

// ── Completions scope filter ────────────────────────────────────────────

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

    async getItems(scope?: ScopeValue): Promise<LibraryListData> {
        const selectedScope = normalizeScope(scope);
        const payload = await fetchJson('/data/items/index.json');

        // Static export writes items/index.json as a raw array.
        const items = Array.isArray(payload)
            ? (payload as LibraryListItem[])
            : (payload as LibraryListData).items ?? [];

        return { items: filterItemsByScope(items, selectedScope) };
    }

    async getItem(id: string): Promise<LibraryDetailData> {
        return (await fetchJson(
            `/data/items/${id}.json`,
        )) as LibraryDetailData;
    }

    async getReadingSummary(scope: ScopeValue): Promise<ReadingSummaryData> {
        const selectedScope = normalizeScope(scope);
        const exported = await this.fetchCached<ExportReadingSummary>(
            '/data/reading/summary.json',
        );
        return pickScope(exported, selectedScope);
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
            '/data/reading/periods.json',
        );

        const sourceData = exported[source as keyof ExportReadingPeriods];
        const groupData = sourceData[
            groupBy as keyof typeof sourceData
        ] as ExportPeriodsByScope;
        return pickScope(groupData, selectedScope);
    }

    async getReadingCalendar(month: string): Promise<ReadingCalendarData> {
        return (await fetchJson(
            `/data/reading/calendar/${month}.json`,
        )) as ReadingCalendarData;
    }

    async getReadingCompletions(
        scope: ScopeValue,
        params: CompletionsParams,
    ): Promise<ReadingCompletionsData> {
        const selectedScope = normalizeScope(scope);
        const year = params.year ?? new Date().getFullYear();
        const data = (await fetchJson(
            `/data/reading/completions/${year}.json`,
        )) as ReadingCompletionsData;
        return filterCompletionsByScope(data, selectedScope);
    }

    getItemDownloadHref(id: string): string {
        return `/data/items/${id}.json`;
    }

    clearCache(): void {
        this.cache.clear();
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
        const metricNames = metric.split(',').map((m) => m.trim()).filter(Boolean);

        // Determine which month files to load from the periods index.
        const periods = await this.fetchCached<ExportReadingPeriods>(
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
        const allPoints: Array<{ key: string; values: Record<string, number> }> = [];

        for (const mk of relevantMonths) {
            let monthData: ExportMonthMetrics;
            try {
                monthData = await this.fetchCached<ExportMonthMetrics>(
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
                const values: Record<string, number> = {};
                for (const m of metricNames) {
                    values[m] = (scopeMetrics[m as keyof typeof scopeMetrics] as number) ?? 0;
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
