export interface ApiMeta {
    version: string;
    generated_at: string;
}

export interface SiteCapabilities {
    has_books: boolean;
    has_comics: boolean;
    has_statistics: boolean;
    has_recap: boolean;
}

export interface SiteResponse {
    meta: ApiMeta;
    title: string;
    capabilities: SiteCapabilities;
}

export interface RecapIndexResponse {
    available_years: number[];
    latest_year?: number | null;
}
