import type {
    LibraryDetailData,
    LibraryListData,
    ReadingAvailablePeriodsData,
    ReadingCalendarData,
    ReadingCompletionsData,
    ReadingMetricsData,
    ReadingSummaryData,
    SiteData,
} from './contracts';

export type ScopeValue = 'all' | 'books' | 'comics';

export interface CompletionsParams {
    year?: number;
    from?: string;
    to?: string;
    groupBy?: string;
    include?: string;
}

export interface ApiClient {
    getSite(): Promise<SiteData>;
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
    clearCache(): void;
}

export function normalizeScope(scope: ScopeValue | undefined): ScopeValue {
    if (scope === 'books' || scope === 'comics') {
        return scope;
    }
    return 'all';
}
