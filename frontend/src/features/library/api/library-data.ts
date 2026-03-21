// Re-export library types from shared contracts.
export type {
    LibraryContentType,
    LibraryStatus,
    LibrarySeries,
    LibraryListItem,
    LibraryDetailItem,
    LibraryReaderPresentation,
    LibraryAnnotation,
    LibraryCompletions,
    LibraryCompletionEntry,
    LibraryItemStats,
    LibrarySessionStats,
    LibraryDetailStatistics,
    LibraryListData,
    LibraryDetailData,
    ExternalIdentifier,
} from '../../../shared/contracts';

// Feature-specific types (not in backend contracts).

export interface LibraryDetailPreviewItem {
    title: string;
    authors: string[];
    series?: { name: string; index?: string | null } | null;
    description?: string | null;
}

export interface LibraryDetailPreviewResponse {
    item: LibraryDetailPreviewItem;
}
