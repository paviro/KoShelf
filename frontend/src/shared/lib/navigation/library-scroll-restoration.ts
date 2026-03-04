export type LibraryCollection = 'books' | 'comics';

const LIBRARY_SCROLL_SNAPSHOT_STORAGE_KEY = 'koshelf_library_list_scroll_snapshot_v1';

type LibraryListScrollSnapshot = {
    collection: LibraryCollection;
    pathname: string;
    scrollY: number;
};

export type LibraryListReturnState = {
    libraryReturnToList?: true;
    libraryCollection?: LibraryCollection;
};

function isLibraryCollection(value: unknown): value is LibraryCollection {
    return value === 'books' || value === 'comics';
}

function libraryListPath(collection: LibraryCollection): string {
    return `/${collection}`;
}

export function libraryDetailCollectionFromPath(pathname: string): LibraryCollection | null {
    if (pathname.startsWith('/books/')) {
        return 'books';
    }

    if (pathname.startsWith('/comics/')) {
        return 'comics';
    }

    return null;
}

export function isLibraryPathForCollection(
    pathname: string,
    collection: LibraryCollection,
): boolean {
    const collectionPath = libraryListPath(collection);
    return pathname === collectionPath || pathname.startsWith(`${collectionPath}/`);
}

function clearSnapshot(): void {
    try {
        window.sessionStorage.removeItem(LIBRARY_SCROLL_SNAPSHOT_STORAGE_KEY);
    } catch {
        // Ignore storage failures.
    }
}

export function clearLibraryListScrollSnapshot(): void {
    clearSnapshot();
}

function readSnapshot(): LibraryListScrollSnapshot | null {
    try {
        const raw = window.sessionStorage.getItem(LIBRARY_SCROLL_SNAPSHOT_STORAGE_KEY);
        if (!raw) {
            return null;
        }

        const parsed = JSON.parse(raw) as Partial<LibraryListScrollSnapshot>;
        if (
            !isLibraryCollection(parsed.collection) ||
            typeof parsed.pathname !== 'string' ||
            typeof parsed.scrollY !== 'number' ||
            !Number.isFinite(parsed.scrollY)
        ) {
            clearSnapshot();
            return null;
        }

        return {
            collection: parsed.collection,
            pathname: parsed.pathname,
            scrollY: Math.max(0, Math.floor(parsed.scrollY)),
        };
    } catch {
        clearSnapshot();
        return null;
    }
}

export function saveLibraryListScrollSnapshot(collection: LibraryCollection): void {
    try {
        const snapshot: LibraryListScrollSnapshot = {
            collection,
            pathname: libraryListPath(collection),
            scrollY: Math.max(0, Math.floor(window.scrollY)),
        };
        window.sessionStorage.setItem(
            LIBRARY_SCROLL_SNAPSHOT_STORAGE_KEY,
            JSON.stringify(snapshot),
        );
    } catch {
        // Ignore storage failures.
    }
}

export function consumeLibraryListScrollSnapshot(
    collection: LibraryCollection,
    pathname: string,
): number | null {
    const snapshot = readSnapshot();
    if (!snapshot) {
        return null;
    }

    if (snapshot.collection !== collection || snapshot.pathname !== pathname) {
        clearSnapshot();
        return null;
    }

    clearSnapshot();
    return snapshot.scrollY;
}

export function createLibraryReturnToListState(collection: LibraryCollection): LibraryListReturnState {
    return {
        libraryReturnToList: true,
        libraryCollection: collection,
    };
}

export function isLibraryReturnToListState(
    state: unknown,
    collection: LibraryCollection,
): boolean {
    if (!state || typeof state !== 'object') {
        return false;
    }

    const candidate = state as LibraryListReturnState;
    return candidate.libraryReturnToList === true && candidate.libraryCollection === collection;
}
