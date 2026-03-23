import type {
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

export type ScopeValue = 'all' | 'books' | 'comics';

export interface UpdateItemPayload {
    review_note?: string | null;
    rating?: number | null;
    status?: string;
}

export interface UpdateAnnotationPayload {
    note?: string | null;
    color?: string;
    drawer?: string;
}

export interface CompletionsParams {
    year?: number;
    from?: string;
    to?: string;
    groupBy?: string;
    include?: string;
}

export interface ApiClient {
    getSite(): Promise<SiteData>;
    login(password: string): Promise<void>;
    getSessions(): Promise<SessionInfo[]>;
    revokeSession(sessionId: string): Promise<void>;
    changePassword(currentPassword: string, newPassword: string): Promise<void>;
    logout(): Promise<void>;
    getItems(scope?: ScopeValue): Promise<LibraryListData>;
    getItem(id: string): Promise<LibraryDetailData>;
    getReadingSummary(
        scope: ScopeValue,
        from?: string,
        to?: string,
    ): Promise<ReadingSummaryData>;
    getReadingMetrics(
        scope: ScopeValue,
        metric: string,
        groupBy: string,
        from?: string,
        to?: string,
    ): Promise<ReadingMetricsData>;
    getAvailablePeriods(
        source: string,
        groupBy: string,
        scope: ScopeValue,
    ): Promise<ReadingAvailablePeriodsData>;
    getReadingCalendar(
        month: string,
        scope: ScopeValue,
    ): Promise<ReadingCalendarData>;
    getReadingCompletions(
        scope: ScopeValue,
        params: CompletionsParams,
    ): Promise<ReadingCompletionsData>;
    getItemDownloadHref(id: string): string;
    getItemFileHref(id: string, format?: string | null): string | null;
    updateItem(id: string, payload: UpdateItemPayload): Promise<void>;
    updateAnnotation(
        itemId: string,
        annotationId: string,
        payload: UpdateAnnotationPayload,
    ): Promise<void>;
    deleteAnnotation(itemId: string, annotationId: string): Promise<void>;
    clearCache(): void;
}

export function normalizeScope(scope: ScopeValue | undefined): ScopeValue {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}
