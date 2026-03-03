import type { ApiMeta } from '../../../shared/contracts';

export type LibraryContentType = 'book' | 'comic';

export type LibraryStatus = 'reading' | 'complete' | 'abandoned' | 'unknown';

export interface LibraryListItem {
    id: string;
    title: string;
    authors: string[];
    series?: string | null;
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
    series?: string | null;
    description?: string | null;
}

export interface LibraryDetailPreviewResponse {
    item: LibraryDetailPreviewItem;
}
