import type { ApiMeta } from '../../../shared/contracts';

export type LibraryContentType = 'book' | 'comic';

export type LibraryStatus = 'reading' | 'complete' | 'abandoned' | 'unknown';

export interface LibrarySeries {
    name: string;
    index?: string | null;
}

export interface LibraryListItem {
    id: string;
    title: string;
    authors: string[];
    series?: LibrarySeries | null;
    status: LibraryStatus;
    progress_percentage?: number | null;
    rating?: number | null;
    annotation_count?: number;
    cover_url: string;
    content_type: LibraryContentType;
}

export interface LibraryListResponse {
    meta: ApiMeta;
    items: LibraryListItem[];
}

export interface LibraryDetailPreviewItem {
    title: string;
    authors: string[];
    series?: LibrarySeries | null;
    description?: string | null;
}

export interface LibraryDetailPreviewResponse {
    item: LibraryDetailPreviewItem;
}

export interface ExternalIdentifier {
    scheme: string;
    value: string;
    display_scheme: string;
    url?: string | null;
}

export interface LibraryDetailItem {
    id: string;
    title: string;
    authors: string[];
    series?: LibrarySeries | null;
    status: LibraryStatus;
    progress_percentage?: number | null;
    rating?: number | null;
    cover_url: string;
    content_type: LibraryContentType;
    language?: string | null;
    publisher?: string | null;
    description?: string | null;
    review_note?: string | null;
    pages?: number | null;
    search_base_path: string;
    subjects: string[];
    identifiers: ExternalIdentifier[];
}

export interface LibraryAnnotation {
    chapter?: string | null;
    datetime?: string | null;
    pageno?: number | null;
    text?: string | null;
    note?: string | null;
}

export interface LibraryCompletionEntry {
    start_date: string;
    end_date: string;
    reading_time_sec: number;
    session_count: number;
    pages_read: number;
}

export interface LibraryCompletions {
    entries: LibraryCompletionEntry[];
    total_completions: number;
    last_completion_date?: string | null;
}

export interface LibraryItemStats {
    notes?: number | null;
    last_open_at?: string | null;
    highlights?: number | null;
    pages?: number | null;
    total_reading_time_sec?: number | null;
}

export interface LibrarySessionStats {
    session_count: number;
    average_session_duration_sec?: number | null;
    longest_session_duration_sec?: number | null;
    last_read_date?: string | null;
    reading_speed?: number | null;
}

export interface LibraryDetailStatistics {
    item_stats?: LibraryItemStats | null;
    session_stats?: LibrarySessionStats | null;
}

export interface LibraryDetailResponse {
    meta: ApiMeta;
    item: LibraryDetailItem;
    highlights?: LibraryAnnotation[] | null;
    bookmarks?: LibraryAnnotation[] | null;
    statistics?: LibraryDetailStatistics | null;
    completions?: LibraryCompletions | null;
}
