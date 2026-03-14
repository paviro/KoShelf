import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';

import { api } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { useRecapCoverTiltEffect } from '../../../shared/lib/dom/useTiltEffect';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageErrorState } from '../../../shared/ui/feedback/PageErrorState';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';
import { RecapHeaderControls } from '../components/RecapHeaderControls';
import { RecapShareModal } from '../components/RecapShareModal';
import {
    useRecapIndexQuery,
    useRecapYearQuery,
} from '../hooks/useRecapQueries';
import {
    orderRecapMonths,
    persistRecapViewState,
    persistRecapSortNewest,
    readStoredRecapScope,
    readStoredRecapYear,
    readRecapSortNewest,
    resolveLatestYear,
} from '../model/recap-model';
import { RecapEmptyState } from '../sections/RecapEmptyState';
import { RecapSummarySection } from '../sections/RecapSummarySection';
import { RecapTimelineSection } from '../sections/RecapTimelineSection';

export function RecapRoute() {
    const [scope, setScope] = useState(() => readStoredRecapScope());
    const [selectedYear, setSelectedYear] = useState<number | null>(() =>
        readStoredRecapYear(),
    );
    const [sortNewestFirst, setSortNewestFirst] = useState(() =>
        readRecapSortNewest(),
    );
    const [shareModalOpenKey, setShareModalOpenKey] = useState<string | null>(
        null,
    );

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.getSite(),
    });

    const recapIndexQuery = useRecapIndexQuery(scope);
    const recapIndexTransition = useQueryTransitionState({
        data: recapIndexQuery.data,
        isLoading: recapIndexQuery.isLoading,
        isFetching: recapIndexQuery.isFetching,
        isPlaceholderData: recapIndexQuery.isPlaceholderData,
    });
    const recapIndex = recapIndexTransition.displayData;
    const availableYears = useMemo(
        () => [...(recapIndex?.available_years ?? [])].reverse(),
        [recapIndex?.available_years],
    );
    const latestYear = resolveLatestYear(
        availableYears,
        recapIndex?.latest_year,
    );
    const yearForQuery = useMemo(() => {
        if (selectedYear !== null && availableYears.includes(selectedYear)) {
            return selectedYear;
        }

        return latestYear;
    }, [availableYears, latestYear, selectedYear]);

    const shareResetKey = `${scope}:${yearForQuery}`;
    const shareModalOpen = shareModalOpenKey === shareResetKey;

    const recapYearQuery = useRecapYearQuery(scope, yearForQuery);
    const recapYearTransition = useQueryTransitionState({
        data: recapYearQuery.data,
        enabled: yearForQuery !== null,
        isLoading: recapYearQuery.isLoading,
        isFetching: recapYearQuery.isFetching,
        isPlaceholderData: recapYearQuery.isPlaceholderData,
    });
    const recapYear = recapYearTransition.displayData;
    const shareAssets = recapYear?.share_assets ?? null;

    const orderedMonths = useMemo(
        () => orderRecapMonths(recapYear?.months ?? [], sortNewestFirst),
        [recapYear?.months, sortNewestFirst],
    );
    const visibleItemsKey = useMemo(
        () =>
            orderedMonths
                .map(
                    (month) =>
                        `${month.key}:${month.items.map((item) => item.end_date).join('|')}`,
                )
                .join('||'),
        [orderedMonths],
    );

    useRecapCoverTiltEffect(
        `${scope}:${yearForQuery ?? 'none'}:${sortNewestFirst}:${visibleItemsKey}`,
    );

    useEffect(() => {
        if (!recapIndexQuery.isSuccess) {
            return;
        }

        persistRecapViewState({
            scope,
            year: yearForQuery,
        });
    }, [recapIndexQuery.isSuccess, scope, yearForQuery]);

    useEffect(() => {
        if (!siteQuery.data?.title) {
            return;
        }

        const titleYear = yearForQuery ?? recapYear?.year;
        const yearSuffix = titleYear ? ` ${titleYear}` : '';
        document.title = `${translation.get('recap')}${yearSuffix} - ${siteQuery.data.title}`;
    }, [recapYear?.year, siteQuery.data?.title, yearForQuery]);

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books &&
        siteQuery.data?.capabilities.has_comics,
    );

    const showPageLevelEmptyState =
        recapIndexTransition.hasFreshData &&
        yearForQuery === null &&
        availableYears.length === 0;
    const showYearEmptyState =
        yearForQuery !== null &&
        recapYearTransition.hasFreshData &&
        !recapYearQuery.isError &&
        recapYear !== null &&
        recapYear.months.length === 0;
    const showTimeline =
        yearForQuery !== null &&
        !recapYearQuery.isError &&
        recapYear !== null &&
        recapYear.months.length > 0;

    return (
        <>
            <PageHeader
                title={translation.get('recap')}
                controls={
                    <RecapHeaderControls
                        showTypeFilter={showTypeFilter}
                        scope={scope}
                        years={availableYears}
                        selectedYear={yearForQuery}
                        onSelectYear={(nextYear) => {
                            setSelectedYear(nextYear);
                            window.scrollTo({
                                top: 0,
                                left: 0,
                                behavior: 'auto',
                            });
                        }}
                        onScopeChange={(nextScope) => {
                            setScope(nextScope);
                            window.scrollTo({
                                top: 0,
                                left: 0,
                                behavior: 'auto',
                            });
                        }}
                        sortNewestFirst={sortNewestFirst}
                        onToggleSort={() => {
                            setSortNewestFirst((current) => {
                                const next = !current;
                                persistRecapSortNewest(next);
                                return next;
                            });
                        }}
                        shareEnabled={Boolean(shareAssets)}
                        onShareClick={() => setShareModalOpenKey(shareResetKey)}
                    />
                }
            />

            <PageContent className="space-y-6 md:space-y-8">
                {!recapIndexQuery.isError &&
                    recapIndexTransition.showBlockingSpinner && (
                        <section className="page-centered-state">
                            <LoadingSpinner size="lg" srLabel="Loading recap" />
                        </section>
                    )}

                {recapIndexQuery.isError && (
                    <PageErrorState
                        error={recapIndexQuery.error}
                        onRetry={() => recapIndexQuery.refetch()}
                    />
                )}

                {!recapIndexQuery.isError && recapIndex && (
                    <div className="relative space-y-6 md:space-y-8">
                        {recapIndexTransition.showOverlaySpinner && (
                            <div className="absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                <LoadingSpinner
                                    size="md"
                                    srLabel="Loading recap"
                                />
                            </div>
                        )}

                        {showPageLevelEmptyState && (
                            <RecapEmptyState hasYearContext={false} />
                        )}

                        {yearForQuery !== null &&
                            recapYearTransition.showBlockingSpinner && (
                                <section className="page-centered-state">
                                    <LoadingSpinner
                                        size="lg"
                                        srLabel="Loading recap year"
                                    />
                                </section>
                            )}

                        {yearForQuery !== null && recapYearQuery.isError && (
                            <PageErrorState
                                error={recapYearQuery.error}
                                onRetry={() => recapYearQuery.refetch()}
                            />
                        )}

                        {showYearEmptyState && (
                            <RecapEmptyState hasYearContext={true} />
                        )}

                        {showTimeline && recapYear && (
                            <div className="relative">
                                {recapYearTransition.showOverlaySpinner && (
                                    <div className="absolute inset-0 z-10 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                        <LoadingSpinner
                                            size="md"
                                            srLabel="Loading recap year"
                                        />
                                    </div>
                                )}
                                <div
                                    className="recap-timeline space-y-6"
                                    id="recapTimeline"
                                >
                                    {recapYear.summary && (
                                        <RecapSummarySection
                                            year={recapYear.year}
                                            scope={scope}
                                            summary={recapYear.summary}
                                        />
                                    )}
                                    <RecapTimelineSection
                                        months={orderedMonths}
                                        scope={scope}
                                    />
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </PageContent>

            <RecapShareModal
                open={shareModalOpen}
                onClose={() => setShareModalOpenKey(null)}
                year={
                    recapYear?.year ?? yearForQuery ?? new Date().getFullYear()
                }
                shareAssets={shareAssets}
            />
        </>
    );
}
