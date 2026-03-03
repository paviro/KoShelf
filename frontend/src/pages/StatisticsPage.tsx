import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useParams } from 'react-router-dom';

import { PageContent } from '../features/layout/PageContent';
import { PageHeader } from '../features/layout/PageHeader';
import { OverallStatsSection } from '../features/statistics/components/OverallStatsSection';
import { ReadingStreakSection } from '../features/statistics/components/ReadingStreakSection';
import { ScopeFilter } from '../features/statistics/components/ScopeFilter';
import { WeeklyStatsSection } from '../features/statistics/components/WeeklyStatsSection';
import { YearlyStatsSection } from '../features/statistics/components/YearlyStatsSection';
import { usePersistentSectionState } from '../features/statistics/hooks/usePersistentSectionState';
import {
    useStatisticsIndexQuery,
    useStatisticsWeekQuery,
    useStatisticsYearQuery,
} from '../features/statistics/hooks/useStatisticsQueries';
import {
    aggregateMonthlyStats,
    isCurrentStreakActive,
    normalizeScope,
    summarizeYearlyStats,
} from '../features/statistics/model/statistics-model';
import { api } from '../shared/api';
import { LoadingSpinner } from '../shared/components/LoadingSpinner';
import type { SiteResponse } from '../shared/contracts';
import type {
    StatisticsWeekResponse,
    StatisticsYearResponse,
} from '../shared/statistics-data-loader';
import { translation } from '../shared/i18n';

const EMPTY_WEEKLY_STATS: StatisticsWeekResponse = {
    week_key: '',
    start_date: '',
    end_date: '',
    read_time: 0,
    pages_read: 0,
    avg_pages_per_day: 0,
    avg_read_time_per_day: 0,
    longest_session_duration: null,
    average_session_duration: null,
};

const PAGE_SPINNER_DELAY_MS = 120;

export function StatisticsPage() {
    const params = useParams();
    const scope = normalizeScope(params.scope);
    const hasInitializedScopeRef = useRef(false);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const statsIndexQuery = useStatisticsIndexQuery(scope);
    const { state: sectionState, toggle: toggleSection } = usePersistentSectionState(scope);

    const availableYears = statsIndexQuery.data?.available_years ?? [];
    const availableWeeks = statsIndexQuery.data?.available_weeks ?? [];

    const [selectedWeekKey, setSelectedWeekKey] = useState<string | null>(null);
    const [selectedHeatmapYear, setSelectedHeatmapYear] = useState<number | null>(null);
    const [selectedYearlyYear, setSelectedYearlyYear] = useState<number | null>(null);
    const [displayedWeeklyStats, setDisplayedWeeklyStats] =
        useState<StatisticsWeekResponse>(EMPTY_WEEKLY_STATS);
    const [displayedYearlyData, setDisplayedYearlyData] = useState<StatisticsYearResponse | null>(
        null,
    );
    const [showPageLoadingSpinner, setShowPageLoadingSpinner] = useState(false);

    const effectiveSelectedWeekKey = selectedWeekKey ?? availableWeeks[0]?.week_key ?? null;
    const effectiveSelectedHeatmapYear = selectedHeatmapYear ?? availableYears[0] ?? null;
    const effectiveSelectedYearlyYear = selectedYearlyYear ?? availableYears[0] ?? null;

    useEffect(() => {
        if (!selectedWeekKey && availableWeeks.length > 0) {
            setSelectedWeekKey(availableWeeks[0].week_key);
        }
        if (!selectedHeatmapYear && availableYears.length > 0) {
            setSelectedHeatmapYear(availableYears[0]);
        }
        if (!selectedYearlyYear && availableYears.length > 0) {
            setSelectedYearlyYear(availableYears[0]);
        }
    }, [availableWeeks, availableYears, selectedHeatmapYear, selectedWeekKey, selectedYearlyYear]);

    useEffect(() => {
        if (!hasInitializedScopeRef.current) {
            hasInitializedScopeRef.current = true;
            return;
        }

        setSelectedWeekKey(null);
        setSelectedHeatmapYear(null);
        setSelectedYearlyYear(null);
        setDisplayedWeeklyStats(EMPTY_WEEKLY_STATS);
        setDisplayedYearlyData(null);
    }, [scope]);

    useEffect(() => {
        document.body.dataset.sectionToggleScope = 'statistics';
        document.body.dataset.sectionToggleKind = scope;

        return () => {
            delete document.body.dataset.sectionToggleScope;
            delete document.body.dataset.sectionToggleKind;
        };
    }, [scope]);

    useEffect(() => {
        if (siteQuery.data?.title) {
            document.title = `${translation.get('reading-statistics')} - ${siteQuery.data.title}`;
        }
    }, [siteQuery.data]);

    const weekQuery = useStatisticsWeekQuery(scope, effectiveSelectedWeekKey);
    const heatmapYearQuery = useStatisticsYearQuery(scope, effectiveSelectedHeatmapYear);
    const yearlyQuery = useStatisticsYearQuery(scope, effectiveSelectedYearlyYear);
    const effectiveDisplayedYearlyData = displayedYearlyData ?? yearlyQuery.data ?? null;

    const statsIndex = statsIndexQuery.data;
    const weeklyLoading =
        weekQuery.isFetching && (displayedWeeklyStats.week_key !== '' || weekQuery.isFetched);

    const yearlyMonthlyStats = useMemo(
        () => aggregateMonthlyStats(effectiveDisplayedYearlyData?.daily_activity ?? []),
        [effectiveDisplayedYearlyData],
    );

    const yearlySummary = useMemo(
        () =>
            summarizeYearlyStats(
                yearlyMonthlyStats,
                effectiveDisplayedYearlyData?.summary.completed_count ?? 0,
            ),
        [yearlyMonthlyStats, effectiveDisplayedYearlyData],
    );

    const validatedCurrentStreak = useMemo(() => {
        const streak = statsIndex?.streaks.current;
        if (!streak) {
            return { days: 0, start_date: null as string | null, end_date: null as string | null };
        }

        if (streak.end_date && !isCurrentStreakActive(streak.end_date)) {
            return { days: 0, start_date: null, end_date: null };
        }

        return streak;
    }, [statsIndex]);

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books && siteQuery.data?.capabilities.has_comics,
    );
    const showPageEmptyState =
        Boolean(statsIndex) && availableYears.length === 0 && availableWeeks.length === 0;
    const yearlyLoading = yearlyQuery.isFetching && effectiveDisplayedYearlyData === null;

    useEffect(() => {
        if (weekQuery.data) {
            setDisplayedWeeklyStats(weekQuery.data);
        }
    }, [weekQuery.data]);

    useEffect(() => {
        if (yearlyQuery.data) {
            setDisplayedYearlyData(yearlyQuery.data);
        }
    }, [yearlyQuery.data]);

    useEffect(() => {
        let timerId: number | null = null;

        if (statsIndexQuery.isLoading) {
            timerId = window.setTimeout(() => {
                setShowPageLoadingSpinner(true);
            }, PAGE_SPINNER_DELAY_MS);
        } else {
            setShowPageLoadingSpinner(false);
        }

        return () => {
            if (timerId !== null) {
                window.clearTimeout(timerId);
            }
        };
    }, [statsIndexQuery.isLoading]);

    const weeklyStats = displayedWeeklyStats;

    return (
        <>
            <PageHeader
                title={translation.get('reading-statistics')}
                controls={<ScopeFilter showTypeFilter={showTypeFilter} scope={scope} />}
            />

            <PageContent className="space-y-6 md:space-y-8">
                {statsIndexQuery.isLoading && showPageLoadingSpinner && (
                    <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                        <LoadingSpinner size="lg" srLabel="Loading statistics" />
                    </section>
                )}

                {statsIndexQuery.isError && (
                    <section className="bg-white dark:bg-dark-850/50 rounded-lg p-6 border border-gray-200/30 dark:border-dark-700/70">
                        <p className="text-sm text-red-600 dark:text-red-400">
                            Failed to load statistics data.
                        </p>
                    </section>
                )}

                {statsIndex && (
                    <>
                        {showPageEmptyState ? (
                            <section className="bg-white dark:bg-dark-850/50 rounded-lg p-8 border border-dashed border-gray-300/80 dark:border-dark-700 text-center">
                                <p className="text-sm text-gray-500 dark:text-dark-300">
                                    {translation.get('stats-empty.nothing-here')}
                                </p>
                            </section>
                        ) : (
                            <>
                                <OverallStatsSection
                                    visible={sectionState['overall-stats']}
                                    onToggle={toggleSection}
                                    overview={statsIndex.overview}
                                />

                                <ReadingStreakSection
                                    visible={sectionState['reading-streak']}
                                    onToggle={toggleSection}
                                    availableYears={availableYears}
                                    selectedYear={effectiveSelectedHeatmapYear}
                                    onSelectYear={setSelectedHeatmapYear}
                                    yearData={heatmapYearQuery.data}
                                    currentStreak={validatedCurrentStreak}
                                    longestStreak={statsIndex.streaks.longest}
                                />

                                <YearlyStatsSection
                                    visible={sectionState['yearly-stats']}
                                    onToggle={toggleSection}
                                    availableYears={availableYears}
                                    selectedYear={effectiveSelectedYearlyYear}
                                    onSelectYear={setSelectedYearlyYear}
                                    yearlySummary={yearlySummary}
                                    yearlyMonthlyStats={yearlyMonthlyStats}
                                    isFetching={yearlyLoading}
                                />

                                <WeeklyStatsSection
                                    visible={sectionState['weekly-stats']}
                                    onToggle={toggleSection}
                                    availableWeeks={availableWeeks}
                                    selectedWeekKey={effectiveSelectedWeekKey}
                                    onSelectWeek={setSelectedWeekKey}
                                    weeklyStats={weeklyStats}
                                    loading={weeklyLoading}
                                />
                            </>
                        )}
                    </>
                )}
            </PageContent>
        </>
    );
}
