import { useQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';

import { listRouteIdForCollection } from '../../../app/routes/route-registry';
import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { useBookCardTiltEffect } from '../../../shared/lib/dom/useTiltEffect';
import {
    patchRouteState,
    readRouteState,
} from '../../../shared/lib/state/route-state-storage';
import { useSectionVisibilityState } from '../../../shared/lib/state/useSectionVisibilityState';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { LibraryEmptyState } from '../components/LibraryEmptyState';
import { LibraryHeader } from '../components/LibraryHeader';
import { LibrarySection } from '../components/LibrarySection';
import { useLibraryHoverPreviewEffect } from '../hooks/useLibraryHoverPreviewEffect';
import { useLibraryListQuery } from '../hooks/useLibraryQueries';
import {
    LIBRARY_FILTER_VALUES,
    LIBRARY_SECTION_KEYS,
    bucketLibraryItems,
    defaultLibrarySectionState,
    itemMatchesSearch,
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
    const routeId = useMemo(
        () => listRouteIdForCollection(collection),
        [collection],
    );

    const desktopSearchInputRef = useRef<HTMLInputElement>(null);
    const mobileSearchInputRef = useRef<HTMLInputElement>(null);

    const [searchTerm, setSearchTerm] = useState(() => {
        const persistedState = readRouteState(routeId, 'session');
        const persisted = persistedState.searchTerm;
        const searchFromQuery = new URLSearchParams(location.search).get(
            'search',
        );
        if (typeof searchFromQuery === 'string') {
            return searchFromQuery;
        }
        return typeof persisted === 'string' ? persisted : '';
    });
    const [mobileSearchOpen, setMobileSearchOpen] = useState(() => {
        const persistedState = readRouteState(routeId, 'session');
        const persisted = persistedState.searchTerm;
        const searchFromQuery = new URLSearchParams(location.search).get(
            'search',
        );
        const initialSearchTerm =
            typeof searchFromQuery === 'string'
                ? searchFromQuery
                : typeof persisted === 'string'
                  ? persisted
                  : '';
        return initialSearchTerm.trim().length > 0 && window.innerWidth < 640;
    });
    const [filterValue, setFilterValue] = useState<LibraryFilterValue>(() => {
        const persisted = readRouteState(routeId, 'session').filterValue;
        return normalizeLibraryFilterValue(
            typeof persisted === 'string' ? persisted : null,
            true,
        );
    });

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });
    const listQuery = useLibraryListQuery(collection);
    const listTransition = useQueryTransitionState({
        data: listQuery.data,
        isLoading: listQuery.isLoading,
        isFetching: listQuery.isFetching,
        isPlaceholderData: listQuery.isPlaceholderData,
    });
    const listData = listTransition.displayData;

    const sectionBuckets = useMemo(
        () => bucketLibraryItems(listData?.items ?? []),
        [listData?.items],
    );

    const hasUnreadItems = sectionBuckets.unread.length > 0;
    const sectionDefaults = useMemo(() => defaultLibrarySectionState(), []);
    const { state: sectionState, toggle: toggleSection } =
        useSectionVisibilityState<LibrarySectionKey>({
            routeId,
            sectionKeys: LIBRARY_SECTION_KEYS,
            defaults: sectionDefaults,
        });

    const filterOptions = useMemo<LibraryFilterValue[]>(() => {
        if (hasUnreadItems) {
            return [...LIBRARY_FILTER_VALUES];
        }

        return LIBRARY_FILTER_VALUES.filter((filter) => filter !== 'unread');
    }, [hasUnreadItems]);

    const effectiveFilterValue: LibraryFilterValue =
        filterValue === 'unread' && !hasUnreadItems ? 'all' : filterValue;
    const handleSearchTermChange = useCallback((nextSearchTerm: string) => {
        setSearchTerm(nextSearchTerm);
        window.scrollTo({ top: 0, left: 0, behavior: 'auto' });
    }, []);
    const handleFilterChange = useCallback((nextFilter: LibraryFilterValue) => {
        setFilterValue(nextFilter);
        window.scrollTo({ top: 0, left: 0, behavior: 'auto' });
    }, []);

    useEffect(() => {
        patchRouteState(routeId, 'session', {
            filterValue: effectiveFilterValue,
        });
    }, [effectiveFilterValue, routeId]);

    useEffect(() => {
        patchRouteState(routeId, 'session', { searchTerm });
    }, [routeId, searchTerm]);

    const querySearchParam = useMemo(() => {
        const query = new URLSearchParams(location.search);
        return query.get('search');
    }, [location.search]);
    if (querySearchParam !== null && querySearchParam !== searchTerm) {
        setSearchTerm(querySearchParam);
    }
    if (
        querySearchParam !== null &&
        querySearchParam.trim().length > 0 &&
        window.innerWidth < 640 &&
        !mobileSearchOpen
    ) {
        setMobileSearchOpen(true);
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

            if (
                event.key === '/' &&
                !event.ctrlKey &&
                !event.metaKey &&
                !event.altKey &&
                !typing
            ) {
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
                    handleSearchTermChange('');
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
                    handleFilterChange('all');
                    break;
                case '2':
                    event.preventDefault();
                    handleFilterChange('reading');
                    break;
                case '3':
                    event.preventDefault();
                    handleFilterChange('completed');
                    break;
                case '4':
                    event.preventDefault();
                    handleFilterChange('abandoned');
                    break;
                case '5':
                    if (!hasUnreadItems) {
                        break;
                    }
                    event.preventDefault();
                    handleFilterChange('unread');
                    break;
                default:
                    break;
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [
        handleFilterChange,
        handleSearchTermChange,
        hasUnreadItems,
        mobileSearchOpen,
        searchTerm,
    ]);

    const normalizedSearch = useMemo(
        () => normalizeSearchTerm(searchTerm),
        [searchTerm],
    );

    const sectionRows = useMemo(
        () =>
            LIBRARY_SECTION_KEYS.map((sectionKey) => {
                const baseItems = sectionBuckets[sectionKey];

                if (!sectionMatchesFilter(sectionKey, effectiveFilterValue)) {
                    return { sectionKey, items: [] };
                }

                const items = baseItems.filter((item) =>
                    itemMatchesSearch(item, normalizedSearch),
                );
                return { sectionKey, items };
            }),
        [effectiveFilterValue, normalizedSearch, sectionBuckets],
    );

    const visibleCardKey = useMemo(
        () =>
            sectionRows
                .flatMap((section) =>
                    section.items.map(
                        (item) => `${section.sectionKey}:${item.id}`,
                    ),
                )
                .join('|'),
        [sectionRows],
    );

    useBookCardTiltEffect(`${collection}:${visibleCardKey}`);
    useLibraryHoverPreviewEffect(`${collection}:${visibleCardKey}`);

    const visibleItemCount = useMemo(
        () =>
            sectionRows.reduce((sum, section) => sum + section.items.length, 0),
        [sectionRows],
    );

    const pageTitle = translation.get(libraryTitleTranslationKey(collection));

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
                onSearchTermChange={handleSearchTermChange}
                filterValue={effectiveFilterValue}
                filterOptions={filterOptions}
                onFilterChange={handleFilterChange}
                mobileSearchOpen={mobileSearchOpen}
                onOpenMobileSearch={() => setMobileSearchOpen(true)}
                onCloseMobileSearch={() => {
                    setMobileSearchOpen(false);
                    handleSearchTermChange('');
                }}
                desktopSearchInputRef={desktopSearchInputRef}
                mobileSearchInputRef={mobileSearchInputRef}
            />

            <PageContent className="space-y-6 md:space-y-8">
                {!listQuery.isError && listTransition.showBlockingSpinner && (
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

                {!listQuery.isError && listTransition.hasDisplayData && (
                    <div className="relative space-y-6 md:space-y-8">
                        {listTransition.showOverlaySpinner && (
                            <div className="absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                <LoadingSpinner size="md" srLabel="Loading library" />
                            </div>
                        )}

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
                                            SECTION_TITLE_KEYS[
                                                section.sectionKey
                                            ],
                                        )}
                                        items={section.items}
                                        collection={collection}
                                        visible={visible}
                                        onToggle={() =>
                                            toggleSection(section.sectionKey)
                                        }
                                    />
                                );
                            })
                        )}
                    </div>
                )}
            </PageContent>
        </>
    );
}
