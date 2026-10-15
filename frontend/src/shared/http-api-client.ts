import { appUrl } from './app-url';
import { translation } from './i18n';
import { addToast } from './ui/toast';
import type {
    ApiClient,
    MetadataPatch,
    CompletionsParams,
    ScopeValue,
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from './api-client';
import { normalizeScope } from './api-client';
import { fetchJson } from './api-fetch';
import type {
    ApiResponse,
    PageActivityData,
    LibraryDetailData,
    LibraryListData,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
    SessionInfo,
    SiteData,
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
    async getSite(): Promise<SiteData> {
        const response = (await fetchJson(
            '/api/site',
        )) as ApiResponse<SiteData>;
        return {
            ...response.data,
            version: response.meta.version,
            generated_at: response.meta.generated_at,
        };
    }

    async login(password: string): Promise<void> {
        await fetchJson('/api/auth/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ password }),
            redirectOnUnauthorized: false,
        });
    }

    async getSessions(): Promise<SessionInfo[]> {
        const response = (await fetchJson('/api/auth/sessions')) as ApiResponse<
            SessionInfo[]
        >;
        return response.data;
    }

    async revokeSession(sessionId: string): Promise<void> {
        await fetchJson(`/api/auth/sessions/${encodeURIComponent(sessionId)}`, {
            method: 'DELETE',
        });
    }

    async changePassword(
        currentPassword: string,
        newPassword: string,
    ): Promise<void> {
        await fetchJson('/api/auth/password', {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                current_password: currentPassword,
                new_password: newPassword,
            }),
        });
    }

    async logout(): Promise<void> {
        await fetchJson('/api/auth/logout', {
            method: 'POST',
        });
    }

    async getItems(scope?: ScopeValue): Promise<LibraryListData> {
        const selectedScope = normalizeScope(scope);
        const response = (await fetchJson(
            appendParams('/api/items', { scope: selectedScope }),
        )) as ApiResponse<LibraryListData>;
        return response.data;
    }

    async getItem(id: string): Promise<LibraryDetailData> {
        const response = (await fetchJson(
            `/api/items/${id}?include=all`,
        )) as ApiResponse<LibraryDetailData>;
        return response.data;
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

    async getItemPageActivity(
        id: string,
        completion?: string,
    ): Promise<PageActivityData> {
        const url = appendParams(
            `/api/items/${encodeURIComponent(id)}/page-activity`,
            { completion },
        );
        const response = (await fetchJson(
            url,
        )) as ApiResponse<PageActivityData>;
        return response.data;
    }

    getItemDownloadHref(id: string): string {
        return appUrl(`/api/items/${id}`);
    }

    getItemFileHref(id: string, format?: string | null): string | null {
        if (format == null) return null;
        return appUrl(
            `/assets/files/${encodeURIComponent(id)}${format ? `.${encodeURIComponent(format)}` : ''}`,
        );
    }

    async updateMetadata(
        id: string,
        payload: MetadataPatch,
        revision: string,
    ): Promise<void> {
        await captureAwareMutation(
            `/api/items/${encodeURIComponent(id)}/metadata`,
            {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Edit-Revision': revision,
                },
                body: JSON.stringify(payload),
            },
        );
    }
    async uploadCover(id: string, file: File, revision: string): Promise<void> {
        // The server detects the image format from the bytes.
        await captureAwareMutation(
            `/api/items/${encodeURIComponent(id)}/cover`,
            {
                method: 'PUT',
                headers: { 'X-Edit-Revision': revision },
                body: file,
            },
        );
    }
    async resetCover(id: string, revision: string): Promise<void> {
        await captureAwareMutation(
            `/api/items/${encodeURIComponent(id)}/cover`,
            {
                method: 'DELETE',
                headers: { 'X-Edit-Revision': revision },
            },
        );
    }

    async updateItem(
        id: string,
        payload: UpdateItemPayload,
        revision: string,
    ): Promise<void> {
        await captureAwareMutation(`/api/items/${encodeURIComponent(id)}`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'X-Metadata-Revision': revision,
            },
            body: JSON.stringify(payload),
        });
    }

    async updateAnnotation(
        itemId: string,
        annotationId: string,
        payload: UpdateAnnotationPayload,
        revision: string,
    ): Promise<void> {
        await captureAwareMutation(
            `/api/items/${encodeURIComponent(itemId)}/annotations/${encodeURIComponent(annotationId)}`,
            {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Metadata-Revision': revision,
                },
                body: JSON.stringify(payload),
            },
        );
    }

    async deleteAnnotation(
        itemId: string,
        annotationId: string,
        revision: string,
    ): Promise<void> {
        await captureAwareMutation(
            `/api/items/${encodeURIComponent(itemId)}/annotations/${encodeURIComponent(annotationId)}`,
            { method: 'DELETE', headers: { 'X-Metadata-Revision': revision } },
        );
    }

    clearCache(): void {
        // No client-side cache in HTTP mode.
    }
}

async function captureAwareMutation(
    ...args: Parameters<typeof fetchJson>
): Promise<void> {
    const result = await fetchJson(...args);
    if (
        result &&
        typeof result === 'object' &&
        'backup_pending' in result &&
        result.backup_pending === true
    ) {
        addToast('warning', translation.get('library-backup-pending'));
    }
}
