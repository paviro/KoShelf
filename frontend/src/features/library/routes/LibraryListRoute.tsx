import { useQuery } from '@tanstack/react-query';
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { useLocation, useNavigate, useNavigationType } from 'react-router-dom';

import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { StorageManager } from '../../../shared/storage-manager';
import { useBookCardTiltEffect } from '../../../shared/lib/dom/useTiltEffect';
import {
    consumeLibraryListScrollSnapshot,
    isLibraryReturnToListState,
} from '../../../shared/lib/navigation/library-scroll-restoration';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { LibraryEmptyState } from '../components/LibraryEmptyState';
import { LibraryHeader } from '../components/LibraryHeader';
import { LibrarySection } from '../components/LibrarySection';
import { useLibraryHoverPreviewEffect } from '../hooks/useLibraryHoverPreviewEffect';
import { useLibraryListQuery } from '../hooks/useLibraryQueries';
import { useLibrarySectionState } from '../hooks/useLibrarySectionState';
import {
    LIBRARY_FILTER_VALUES,
    LIBRARY_SECTION_KEYS,
    bucketLibraryItems,
    itemMatchesSearch,
    libraryFilterStorageKey,
    libraryTitleTranslationKey,
    normalizeLibraryFilterValue,
    normalizeSearchTerm,
    sectionMatchesFilter,
    type LibraryCollection,
    type LibraryFilterValue,
    type LibrarySectionKey,
} from '../model/library-model';

type LibraryListRouteProps = {
    collection: LibraryCollection;
};

const SECTION_TITLE_KEYS: Record<LibrarySectionKey, string> = {
    reading: 'status.reading',
    abandoned: 'status.on-hold',
    completed: 'status.completed',
    unread: 'status.unread',
};

function isTypingTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
        return false;
    }

    const tagName = target.tagName;
    return (
        target.isContentEditable ||
        tagName === 'INPUT' ||
        tagName === 'TEXTAREA' ||
        tagName === 'SELECT'
    );
}

export function LibraryListRoute({ collection }: LibraryListRouteProps) {
    const location = useLocation();
    const navigate = useNavigate();
    const navigationType = useNavigationType();

    const desktopSearchInputRef = useRef<HTMLInputElement>(null);
    const mobileSearchInputRef = useRef<HTMLInputElement>(null);
    const restoredLocationRef = useRef<string | null>(null);

    const [searchTerm, setSearchTerm] = useState('');
    const [mobileSearchOpen, setMobileSearchOpen] = useState(false);
    const [filterValue, setFilterValue] = useState<LibraryFilterValue>(() => {
        const persisted = StorageManager.get<string>(libraryFilterStorageKey(collection), 'all');
        return normalizeLibraryFilterValue(persisted, true);
    });

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });
    const listQuery = useLibraryListQuery(collection);

    const sectionBuckets = useMemo(
        () => bucketLibraryItems(listQuery.data?.items ?? []),
        [listQuery.data?.items],
    );

    const hasUnreadItems = sectionBuckets.unread.length > 0;
    const { state: sectionState, toggle: toggleSection } = useLibrarySectionState(collection);

    const filterOptions = useMemo<LibraryFilterValue[]>(() => {
        if (hasUnreadItems) {
            return [...LIBRARY_FILTER_VALUES];
        }

        return LIBRARY_FILTER_VALUES.filter((filter) => filter !== 'unread');
    }, [hasUnreadItems]);

    const [prevCollection, setPrevCollection] = useState(collection);
    if (prevCollection !== collection) {
        setPrevCollection(collection);
        const persisted = StorageManager.get<string>(libraryFilterStorageKey(collection), 'all');
        setFilterValue(normalizeLibraryFilterValue(persisted, true));
        setSearchTerm('');
        setMobileSearchOpen(false);
    }

    if (filterValue === 'unread' && !hasUnreadItems) {
        setFilterValue('all');
    }

    useEffect(() => {
        StorageManager.set(libraryFilterStorageKey(collection), filterValue);
    }, [collection, filterValue]);

    const searchQuery = new URLSearchParams(location.search);
    const querySearchParam = searchQuery.get('search');
    if (querySearchParam !== null) {
        setSearchTerm(querySearchParam);
        if (querySearchParam.trim().length > 0 && window.innerWidth < 640) {
            setMobileSearchOpen(true);
        }
    }

    useEffect(() => {
        const query = new URLSearchParams(location.search);
        if (!query.has('search')) {
            return;
        }
        query.delete('search');
        navigate(
            {
                pathname: location.pathname,
                search: query.toString() ? `?${query.toString()}` : '',
            },
            { replace: true },
        );
    }, [location.pathname, location.search, navigate]);

    useEffect(() => {
        if (!mobileSearchOpen) {
            return;
        }

        const timeoutId = window.setTimeout(() => {
            mobileSearchInputRef.current?.focus();
        }, 50);

        return () => {
            window.clearTimeout(timeoutId);
        };
    }, [mobileSearchOpen]);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent): void => {
            const typing = isTypingTarget(event.target);

            if (event.key === '/' && !event.ctrlKey && !event.metaKey && !event.altKey && !typing) {
                event.preventDefault();

                if (window.innerWidth < 640) {
                    setMobileSearchOpen(true);
                    return;
                }

                desktopSearchInputRef.current?.focus();
                return;
            }

            if (event.key === 'Escape') {
                const activeElement = document.activeElement;
                if (activeElement instanceof HTMLElement) {
                    activeElement.blur();
                }

                if (searchTerm) {
                    setSearchTerm('');
                }

                if (mobileSearchOpen) {
                    setMobileSearchOpen(false);
                }

                return;
            }

            if (!event.altKey || typing) {
                return;
            }

            switch (event.key) {
                case '1':
                    event.preventDefault();
                    setFilterValue('all');
                    break;
                case '2':
                    event.preventDefault();
                    setFilterValue('reading');
                    break;
                case '3':
                    event.preventDefault();
                    setFilterValue('completed');
                    break;
                case '4':
                    event.preventDefault();
                    setFilterValue('abandoned');
                    break;
                case '5':
                    if (!hasUnreadItems) {
                        break;
                    }
                    event.preventDefault();
                    setFilterValue('unread');
                    break;
                default:
                    break;
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [hasUnreadItems, mobileSearchOpen, searchTerm]);

    const normalizedSearch = useMemo(() => normalizeSearchTerm(searchTerm), [searchTerm]);

    const sectionRows = useMemo(
        () =>
            LIBRARY_SECTION_KEYS.map((sectionKey) => {
                const baseItems = sectionBuckets[sectionKey];

                if (!sectionMatchesFilter(sectionKey, filterValue)) {
                    return { sectionKey, items: [] };
                }

                const items = baseItems.filter((item) => itemMatchesSearch(item, normalizedSearch));
                return { sectionKey, items };
            }),
        [filterValue, normalizedSearch, sectionBuckets],
    );

    const visibleCardKey = useMemo(
        () =>
            sectionRows
                .flatMap((section) =>
                    section.items.map((item) => `${section.sectionKey}:${item.id}`),
                )
                .join('|'),
        [sectionRows],
    );

    useBookCardTiltEffect(`${collection}:${visibleCardKey}`);
    useLibraryHoverPreviewEffect(`${collection}:${visibleCardKey}`);

    const visibleItemCount = useMemo(
        () => sectionRows.reduce((sum, section) => sum + section.items.length, 0),
        [sectionRows],
    );

    const pageTitle = translation.get(libraryTitleTranslationKey(collection));

    const shouldAttemptScrollRestore = useMemo(() => {
        const query = new URLSearchParams(location.search);
        if (query.has('search')) {
            return false;
        }

        if (isLibraryReturnToListState(location.state, collection)) {
            return true;
        }

        return navigationType === 'POP';
    }, [collection, location.search, location.state, navigationType]);

    useLayoutEffect(() => {
        if (!shouldAttemptScrollRestore || listQuery.isLoading || listQuery.isError) {
            return;
        }

        const restoreKey = `${collection}:${location.key}`;
        if (restoredLocationRef.current === restoreKey) {
            return;
        }

        restoredLocationRef.current = restoreKey;
        const savedScrollY = consumeLibraryListScrollSnapshot(collection, location.pathname);
        if (savedScrollY === null) {
            return;
        }

        window.scrollTo({ top: savedScrollY, behavior: 'auto' });
    }, [
        collection,
        listQuery.isError,
        listQuery.isLoading,
        location.key,
        location.pathname,
        shouldAttemptScrollRestore,
    ]);

    useEffect(() => {
        if (siteQuery.data?.title) {
            document.title = `${pageTitle} - ${siteQuery.data.title}`;
        }
    }, [pageTitle, siteQuery.data]);

    return (
        <>
            <LibraryHeader
                title={pageTitle}
                searchTerm={searchTerm}
                onSearchTermChange={setSearchTerm}
                filterValue={filterValue}
                filterOptions={filterOptions}
                onFilterChange={setFilterValue}
                mobileSearchOpen={mobileSearchOpen}
                onOpenMobileSearch={() => setMobileSearchOpen(true)}
                onCloseMobileSearch={() => {
                    setMobileSearchOpen(false);
                    setSearchTerm('');
                }}
                desktopSearchInputRef={desktopSearchInputRef}
                mobileSearchInputRef={mobileSearchInputRef}
            />

            <PageContent className="space-y-6 md:space-y-8">
                {listQuery.isLoading && (
                    <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                        <LoadingSpinner size="lg" srLabel="Loading library" />
                    </section>
                )}

                {listQuery.isError && (
                    <section className="bg-white dark:bg-dark-850/50 rounded-lg p-6 border border-gray-200/30 dark:border-dark-700/70">
                        <p className="text-sm text-red-600 dark:text-red-400">
                            Failed to load library data.
                        </p>
                    </section>
                )}

                {!listQuery.isLoading && !listQuery.isError && (
                    <>
                        {visibleItemCount === 0 ? (
                            <LibraryEmptyState />
                        ) : (
                            sectionRows.map((section) => {
                                if (section.items.length === 0) {
                                    return null;
                                }

                                const visible = normalizedSearch
                                    ? true
                                    : sectionState[section.sectionKey];

                                return (
                                    <LibrarySection
                                        key={section.sectionKey}
                                        sectionKey={section.sectionKey}
                                        title={translation.get(
                                            SECTION_TITLE_KEYS[section.sectionKey],
                                        )}
                                        items={section.items}
                                        collection={collection}
                                        visible={visible}
                                        onToggle={() => toggleSection(section.sectionKey)}
                                    />
                                );
                            })
                        )}
                    </>
                )}
            </PageContent>
        </>
    );
}
