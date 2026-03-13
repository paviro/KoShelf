import type { ApiClient, CompletionsParams, ScopeValue } from './api-client';
import { normalizeScope } from './api-client';
import { fetchJson } from './api-fetch';
import type {
    ApiResponse,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
    SiteResponse,
} from './contracts';

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

export class HttpApiClient implements ApiClient {
    async getSite(): Promise<SiteResponse> {
        return (await fetchJson('/api/site')) as SiteResponse;
    }

    async getItems<T = unknown>(scope?: ScopeValue): Promise<T> {
        const selectedScope = normalizeScope(scope);
        return (await fetchJson(
            appendParams('/api/items', { scope: selectedScope }),
        )) as T;
    }

    async getItem<T = unknown>(id: string): Promise<T> {
        return (await fetchJson(`/api/items/${id}?include=all`)) as T;
    }

    async getReadingSummary(
        scope: ScopeValue,
        from?: string,
        to?: string,
    ): Promise<ReadingSummaryData> {
        const selectedScope = normalizeScope(scope);
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

    async getReadingMetrics(
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ): Promise<ReadingMetricsData> {
        const selectedScope = normalizeScope(scope);
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

    async getAvailablePeriods(
        source: string,
        groupBy: string,
        scope: ScopeValue,
    ): Promise<ReadingAvailablePeriodsData> {
        const selectedScope = normalizeScope(scope);
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

    async getReadingCalendar(
        month: string,
        scope: ScopeValue,
    ): Promise<ReadingCalendarData> {
        const selectedScope = normalizeScope(scope);
        const url = appendParams('/api/reading/calendar', {
            month,
            scope: selectedScope,
        });
        const response = (await fetchJson(
            url,
        )) as ApiResponse<ReadingCalendarData>;
        return response.data;
    }

    async getReadingCompletions(
        scope: ScopeValue,
        params: CompletionsParams,
    ): Promise<ReadingCompletionsData> {
        const selectedScope = normalizeScope(scope);
        const url = appendParams('/api/reading/completions', {
            scope: selectedScope,
            year: params.year !== undefined ? String(params.year) : undefined,
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

    getItemDownloadHref(id: string): string {
        return `/api/items/${id}`;
    }

    clearCache(): void {
        // No client-side cache in HTTP mode.
    }
}
