export const LIBRARY_DETAIL_SECTION_KEYS = [
    'book-overview',
    'reading-stats',
    'review',
    'highlights',
    'bookmarks',
    'additional-info',
] as const;

export type LibraryDetailSectionKey = (typeof LIBRARY_DETAIL_SECTION_KEYS)[number];

export type LibraryDetailSectionVisibilityState = Record<LibraryDetailSectionKey, boolean>;

const DEFAULT_DETAIL_SECTION_STATE: LibraryDetailSectionVisibilityState = {
    'book-overview': true,
    'reading-stats': false,
    review: true,
    highlights: true,
    bookmarks: true,
    'additional-info': false,
};

export function defaultLibraryDetailSectionState(): LibraryDetailSectionVisibilityState {
    return { ...DEFAULT_DETAIL_SECTION_STATE };
}
