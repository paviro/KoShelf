import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo } from 'react';
import { Link, Navigate, useLocation, useParams } from 'react-router-dom';

import {
    buildRoutePath,
    detailRouteIdForCollection,
    listRouteIdForCollection,
} from '../../../app/routes/route-registry';
import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { resolveDetailReturnPath } from '../../../shared/lib/navigation/detail-return-state';
import { useSectionVisibilityState } from '../../../shared/lib/state/useSectionVisibilityState';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageErrorState } from '../../../shared/ui/feedback/PageErrorState';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { LibraryDetailHeader } from '../components/LibraryDetailHeader';
import { useLibraryDetailQuery } from '../hooks/useLibraryQueries';
import {
    LIBRARY_DETAIL_SECTION_KEYS,
    defaultLibraryDetailSectionState,
    type LibraryDetailSectionKey,
} from '../model/library-detail-model';
import type { LibraryCollection } from '../model/library-model';
import { LibraryAdditionalInfoSection } from '../sections/LibraryAdditionalInfoSection';
import { LibraryBookmarksSection } from '../sections/LibraryBookmarksSection';
import { LibraryHighlightsSection } from '../sections/LibraryHighlightsSection';
import { LibraryOverviewSection } from '../sections/LibraryOverviewSection';
import { LibraryReadingStatsSection } from '../sections/LibraryReadingStatsSection';
import { LibraryReviewSection } from '../sections/LibraryReviewSection';

type LibraryDetailRouteProps = {
    collection: LibraryCollection;
};

function collectionTitle(collection: LibraryCollection): string {
    return translation.get(collection === 'books' ? 'books' : 'comics');
}

export function LibraryDetailRoute({ collection }: LibraryDetailRouteProps) {
    const params = useParams();
    const location = useLocation();
    const id = params.id;

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const detailQuery = useLibraryDetailQuery(collection, id);
    const detailTransition = useQueryTransitionState({
        data: detailQuery.data,
        enabled: Boolean(id),
        isLoading: detailQuery.isLoading,
        isFetching: detailQuery.isFetching,
        isPlaceholderData: detailQuery.isPlaceholderData,
    });
    const detail = detailTransition.displayData;
    const item = detail?.item;
    const itemStats = detail?.statistics.item_stats ?? null;
    const sessionStats = detail?.statistics.session_stats ?? null;
    const completions =
        detail?.statistics.completions ?? itemStats?.completions ?? null;

    const highlightAnnotations = detail?.highlights ?? [];
    const bookmarkAnnotations = detail?.bookmarks ?? [];

    const noteCount =
        typeof itemStats?.notes === 'number' && Number.isFinite(itemStats.notes)
            ? Math.max(0, Math.floor(itemStats.notes))
            : 0;
    const reviewNote = item?.review_note ?? '';
    const hasReviewNote =
        item?.review_note !== null && item?.review_note !== undefined;
    const hasPublisher =
        item?.publisher !== null && item?.publisher !== undefined;

    const detailRouteId = useMemo(
        () => detailRouteIdForCollection(collection),
        [collection],
    );
    const sectionDefaults = useMemo(
        () => defaultLibraryDetailSectionState(),
        [],
    );
    const { state: sectionState, toggle } =
        useSectionVisibilityState<LibraryDetailSectionKey>({
            routeId: detailRouteId,
            sectionKeys: LIBRARY_DETAIL_SECTION_KEYS,
            defaults: sectionDefaults,
        });

    useEffect(() => {
        if (!siteQuery.data?.title) {
            return;
        }

        if (item?.title) {
            document.title = `${item.title} - ${siteQuery.data.title}`;
            return;
        }

        document.title = `${collectionTitle(collection)} - ${siteQuery.data.title}`;
    }, [collection, item?.title, siteQuery.data?.title]);

    if (!id) {
        return (
            <Navigate
                to={buildRoutePath(listRouteIdForCollection(collection))}
                replace
            />
        );
    }

    const headerTitle = item?.title ?? collectionTitle(collection);
    const primaryAuthor = item?.authors[0];
    const returnTo = resolveDetailReturnPath(location.state);
    const backHref =
        returnTo ?? buildRoutePath(listRouteIdForCollection(collection));

    return (
        <>
            <LibraryDetailHeader
                title={headerTitle}
                primaryAuthor={primaryAuthor}
                collection={collection}
                itemId={id}
                backHref={backHref}
            />

            <PageContent className="space-y-6 md:space-y-8">
                {!detailQuery.isError &&
                    detailTransition.showBlockingSpinner && (
                        <section className="page-centered-state">
                            <LoadingSpinner
                                size="lg"
                                srLabel="Loading item details"
                            />
                        </section>
                    )}

                {detailQuery.isError && (
                    <PageErrorState
                        error={detailQuery.error}
                        onRetry={() => detailQuery.refetch()}
                    >
                        <Link
                            to={backHref}
                            className="inline-flex items-center px-5 py-2.5 rounded-lg text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
                        >
                            {collectionTitle(collection)}
                        </Link>
                    </PageErrorState>
                )}

                {!detailQuery.isError && item && (
                    <div className="relative space-y-6 md:space-y-8">
                        {detailTransition.showOverlaySpinner && (
                            <div className="absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                <LoadingSpinner
                                    size="md"
                                    srLabel="Loading item details"
                                />
                            </div>
                        )}

                        <LibraryOverviewSection
                            item={item}
                            itemStats={itemStats}
                            completions={completions}
                            highlightCount={highlightAnnotations.length}
                            noteCount={noteCount}
                            visible={sectionState['book-overview']}
                            onToggle={() => toggle('book-overview')}
                        />

                        {sessionStats && (
                            <LibraryReadingStatsSection
                                itemStats={itemStats}
                                sessionStats={sessionStats}
                                completions={completions}
                                visible={sectionState['reading-stats']}
                                onToggle={() => toggle('reading-stats')}
                            />
                        )}

                        {hasReviewNote && (
                            <LibraryReviewSection
                                note={reviewNote}
                                rating={item.rating ?? null}
                                visible={sectionState.review}
                                onToggle={() => toggle('review')}
                            />
                        )}

                        {item.content_type === 'book' &&
                            highlightAnnotations.length > 0 && (
                                <LibraryHighlightsSection
                                    annotations={highlightAnnotations}
                                    visible={sectionState.highlights}
                                    onToggle={() => toggle('highlights')}
                                />
                            )}

                        {item.content_type === 'book' &&
                            bookmarkAnnotations.length > 0 && (
                                <LibraryBookmarksSection
                                    annotations={bookmarkAnnotations}
                                    visible={sectionState.bookmarks}
                                    onToggle={() => toggle('bookmarks')}
                                />
                            )}

                        {(hasPublisher || item.identifiers.length > 0) && (
                            <LibraryAdditionalInfoSection
                                publisher={item.publisher ?? null}
                                identifiers={item.identifiers}
                                visible={sectionState['additional-info']}
                                onToggle={() => toggle('additional-info')}
                            />
                        )}
                    </div>
                )}
            </PageContent>
        </>
    );
}
