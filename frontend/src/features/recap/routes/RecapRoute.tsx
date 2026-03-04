import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';

import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { useRecapCoverTiltEffect } from '../../../shared/lib/dom/useTiltEffect';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';
import { RecapHeaderControls } from '../components/RecapHeaderControls';
import { RecapShareModal } from '../components/RecapShareModal';
import { useRecapIndexQuery, useRecapYearQuery } from '../hooks/useRecapQueries';
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
    const [selectedYear, setSelectedYear] = useState<number | null>(() => readStoredRecapYear());
    const [sortNewestFirst, setSortNewestFirst] = useState(() => readRecapSortNewest());
    const [shareModalOpenKey, setShareModalOpenKey] = useState<string | null>(null);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const recapIndexQuery = useRecapIndexQuery(scope);
    const availableYears = useMemo(() => recapIndexQuery.data?.available_years ?? [], [recapIndexQuery.data?.available_years]);
    const latestYear = resolveLatestYear(availableYears, recapIndexQuery.data?.latest_year);
    const yearForQuery = useMemo(() => {
        if (selectedYear !== null && availableYears.includes(selectedYear)) {
            return selectedYear;
        }

        return latestYear;
    }, [availableYears, latestYear, selectedYear]);

    const shareResetKey = `${scope}:${yearForQuery}`;
    const shareModalOpen = shareModalOpenKey === shareResetKey;

    const recapYearQuery = useRecapYearQuery(scope, yearForQuery);
    const recapYear = recapYearQuery.data ?? null;
    const shareAssets = recapYear?.share_assets ?? null;

    const orderedMonths = useMemo(
        () => orderRecapMonths(recapYear?.months ?? [], sortNewestFirst),
        [recapYear?.months, sortNewestFirst],
    );
    const visibleItemsKey = useMemo(
        () =>
            orderedMonths
                .map((month) => `${month.month_key}:${month.items.map((item) => item.end_date).join('|')}`)
                .join('||'),
        [orderedMonths],
    );

    useRecapCoverTiltEffect(`${scope}:${yearForQuery ?? 'none'}:${sortNewestFirst}:${visibleItemsKey}`);

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

        const titleYear = recapYear?.year ?? yearForQuery;
        const yearSuffix = titleYear ? ` ${titleYear}` : '';
        document.title = `${translation.get('recap')}${yearSuffix} - ${siteQuery.data.title}`;
    }, [recapYear?.year, siteQuery.data?.title, yearForQuery]);

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books && siteQuery.data?.capabilities.has_comics,
    );

    const showPageLevelEmptyState =
        !recapIndexQuery.isLoading &&
        !recapIndexQuery.isError &&
        yearForQuery === null &&
        availableYears.length === 0;
    const recapYearLoading =
        recapYearQuery.isFetching && recapYear === null;
    const showYearEmptyState =
        !recapYearLoading &&
        !recapYearQuery.isError &&
        recapYear !== null &&
        recapYear.months.length === 0;
    const showTimeline =
        !recapYearLoading &&
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
                            window.scrollTo({ top: 0, left: 0, behavior: 'auto' });
                        }}
                        onScopeChange={(nextScope) => {
                            setScope(nextScope);
                            window.scrollTo({ top: 0, left: 0, behavior: 'auto' });
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
                {recapIndexQuery.isLoading && (
                    <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                        <LoadingSpinner size="lg" srLabel="Loading recap" />
                    </section>
                )}

                {recapIndexQuery.isError && (
                    <section className="bg-white dark:bg-dark-850/50 rounded-lg p-6 border border-gray-200/30 dark:border-dark-700/70">
                        <p className="text-sm text-red-600 dark:text-red-400">
                            Failed to load recap data.
                        </p>
                    </section>
                )}

                {!recapIndexQuery.isLoading && !recapIndexQuery.isError && (
                    <>
                        {showPageLevelEmptyState && <RecapEmptyState hasYearContext={false} />}

                        {yearForQuery !== null && recapYearLoading && (
                            <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                                <LoadingSpinner size="lg" srLabel="Loading recap year" />
                            </section>
                        )}

                        {yearForQuery !== null && recapYearQuery.isError && (
                            <section className="bg-white dark:bg-dark-850/50 rounded-lg p-6 border border-gray-200/30 dark:border-dark-700/70">
                                <p className="text-sm text-red-600 dark:text-red-400">
                                    Failed to load recap year data.
                                </p>
                            </section>
                        )}

                        {showYearEmptyState && <RecapEmptyState hasYearContext={true} />}

                        {showTimeline && recapYear && (
                            <div className="recap-timeline space-y-6" id="recapTimeline">
                                <RecapSummarySection
                                    year={recapYear.year}
                                    scope={scope}
                                    summary={recapYear.summary}
                                />
                                <RecapTimelineSection months={orderedMonths} scope={scope} />
                            </div>
                        )}
                    </>
                )}
            </PageContent>

            <RecapShareModal
                open={shareModalOpen}
                onClose={() => setShareModalOpenKey(null)}
                year={recapYear?.year ?? yearForQuery ?? new Date().getFullYear()}
                shareAssets={shareAssets}
            />
        </>
    );
}
