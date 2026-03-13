import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import { useLocation } from 'react-router-dom';

import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';
import { OverallStatsSection } from '../sections/OverallStatsSection';
import { ReadingStreakSection } from '../sections/ReadingStreakSection';
import { ScopeFilter } from '../components/ScopeFilter';
import { StatisticsEmptyState } from '../sections/StatisticsEmptyState';
import { WeeklyStatsSection } from '../sections/WeeklyStatsSection';
import { YearlyStatsSection } from '../sections/YearlyStatsSection';
import { useSectionVisibilityState } from '../../../shared/lib/state/useSectionVisibilityState';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import {
    useStatisticsIndexQuery,
    useStatisticsWeekQuery,
    useStatisticsYearQuery,
} from '../hooks/useStatisticsQueries';
import {
    SECTION_NAMES,
    aggregateMonthlyStats,
    defaultSectionState,
    isCurrentStreakActive,
    persistStatisticsViewState,
    readStoredStatisticsViewState,
    summarizeYearlyStats,
    type SectionName,
} from '../model/statistics-model';
import { api } from '../../../shared/api';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageErrorState } from '../../../shared/ui/feedback/PageErrorState';
import type { SiteResponse } from '../../../shared/contracts';
import type { StatisticsWeekResponse } from '../api/statistics-data';
import { translation } from '../../../shared/i18n';

const EMPTY_WEEKLY_STATS: StatisticsWeekResponse = {
    week_key: '',
    start_date: '',
    end_date: '',
    reading_time_sec: 0,
    pages_read: 0,
    longest_session_duration_sec: null,
    average_session_duration_sec: null,
    daily_activity: [],
};

export function StatisticsRoute() {
    const location = useLocation();
    const [initialViewState] = useState(() => readStoredStatisticsViewState());
    const [scope, setScope] = useState(() => initialViewState.scope);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const statsIndexQuery = useStatisticsIndexQuery(scope);
    const statsIndexTransition = useQueryTransitionState({
        data: statsIndexQuery.data,
        isLoading: statsIndexQuery.isLoading,
        isFetching: statsIndexQuery.isFetching,
        isPlaceholderData: statsIndexQuery.isPlaceholderData,
    });
    const statsIndex = statsIndexTransition.displayData;
    const sectionDefaults = useMemo(() => defaultSectionState(), []);
    const { state: sectionState, toggle: toggleSection } =
        useSectionVisibilityState<SectionName>({
            routeId: 'statistics',
            sectionKeys: SECTION_NAMES,
            defaults: sectionDefaults,
        });

    const availableYears = useMemo(
        () => statsIndex?.available_years ?? [],
        [statsIndex?.available_years],
    );
    const availableWeeks = useMemo(
        () => statsIndex?.available_weeks ?? [],
        [statsIndex?.available_weeks],
    );

    const [selectedWeekKey, setSelectedWeekKey] = useState<string | null>(
        () => initialViewState.selectedWeekKey,
    );
    const [selectedHeatmapYear, setSelectedHeatmapYear] = useState<
        number | null
    >(() => initialViewState.selectedHeatmapYear);
    const [selectedYearlyYear, setSelectedYearlyYear] = useState<number | null>(
        () => initialViewState.selectedYearlyYear,
    );

    const effectiveSelectedWeek = useMemo(() => {
        if (selectedWeekKey) {
            const found = availableWeeks.find(
                (week) => week.week_key === selectedWeekKey,
            );
            if (found) return found;
        }

        return availableWeeks[0] ?? null;
    }, [availableWeeks, selectedWeekKey]);
    const effectiveSelectedWeekKey = effectiveSelectedWeek?.week_key ?? null;
    const effectiveSelectedHeatmapYear = useMemo(() => {
        if (
            selectedHeatmapYear !== null &&
            availableYears.includes(selectedHeatmapYear)
        ) {
            return selectedHeatmapYear;
        }

        return availableYears[0] ?? null;
    }, [availableYears, selectedHeatmapYear]);
    const effectiveSelectedYearlyYear = useMemo(() => {
        if (
            selectedYearlyYear !== null &&
            availableYears.includes(selectedYearlyYear)
        ) {
            return selectedYearlyYear;
        }

        return availableYears[0] ?? null;
    }, [availableYears, selectedYearlyYear]);

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

    useEffect(() => {
        if (!statsIndexTransition.hasFreshData) {
            return;
        }

        persistStatisticsViewState({
            scope,
            selectedWeekKey: effectiveSelectedWeekKey,
            selectedHeatmapYear: effectiveSelectedHeatmapYear,
            selectedYearlyYear: effectiveSelectedYearlyYear,
        });
    }, [
        effectiveSelectedHeatmapYear,
        effectiveSelectedWeekKey,
        effectiveSelectedYearlyYear,
        scope,
        statsIndexTransition.hasFreshData,
    ]);

    const weekQuery = useStatisticsWeekQuery(scope, effectiveSelectedWeek);
    const weekTransition = useQueryTransitionState({
        data: weekQuery.data,
        enabled: Boolean(effectiveSelectedWeekKey),
        isLoading: weekQuery.isLoading,
        isFetching: weekQuery.isFetching,
        isPlaceholderData: weekQuery.isPlaceholderData,
    });
    const heatmapYearQuery = useStatisticsYearQuery(
        scope,
        effectiveSelectedHeatmapYear,
    );
    const heatmapYearTransition = useQueryTransitionState({
        data: heatmapYearQuery.data,
        enabled: Boolean(effectiveSelectedHeatmapYear),
        isLoading: heatmapYearQuery.isLoading,
        isFetching: heatmapYearQuery.isFetching,
        isPlaceholderData: heatmapYearQuery.isPlaceholderData,
    });
    const yearlyQuery = useStatisticsYearQuery(
        scope,
        effectiveSelectedYearlyYear,
    );
    const yearlyTransition = useQueryTransitionState({
        data: yearlyQuery.data,
        enabled: Boolean(effectiveSelectedYearlyYear),
        isLoading: yearlyQuery.isLoading,
        isFetching: yearlyQuery.isFetching,
        isPlaceholderData: yearlyQuery.isPlaceholderData,
    });
    const effectiveDisplayedYearlyData = yearlyTransition.displayData;

    const weeklyLoading =
        weekTransition.showBlockingSpinner || weekTransition.showOverlaySpinner;
    const yearlyLoading =
        yearlyTransition.showBlockingSpinner ||
        yearlyTransition.showOverlaySpinner;
    const heatmapLoading =
        heatmapYearTransition.showBlockingSpinner ||
        heatmapYearTransition.showOverlaySpinner;

    const yearlyMonthlyStats = useMemo(
        () =>
            aggregateMonthlyStats(
                effectiveDisplayedYearlyData?.daily_activity ?? [],
            ),
        [effectiveDisplayedYearlyData],
    );

    const yearlySummary = useMemo(
        () =>
            summarizeYearlyStats(
                yearlyMonthlyStats,
                effectiveDisplayedYearlyData?.completions ?? 0,
            ),
        [yearlyMonthlyStats, effectiveDisplayedYearlyData],
    );

    const validatedCurrentStreak = useMemo(() => {
        const streak = statsIndex?.streaks.current;
        if (!streak) {
            return {
                days: 0,
                start_date: null as string | null,
                end_date: null as string | null,
            };
        }

        if (streak.end_date && !isCurrentStreakActive(streak.end_date)) {
            return { days: 0, start_date: null, end_date: null };
        }

        return streak;
    }, [statsIndex]);

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books &&
        siteQuery.data?.capabilities.has_comics,
    );
    const showPageEmptyState =
        statsIndexTransition.hasFreshData &&
        availableYears.length === 0 &&
        availableWeeks.length === 0;
    const weeklyStats = weekTransition.displayData ?? EMPTY_WEEKLY_STATS;

    return (
        <>
            <PageHeader
                title={translation.get('reading-statistics')}
                controls={
                    <ScopeFilter
                        showTypeFilter={showTypeFilter}
                        scope={scope}
                        onScopeChange={(nextScope) => {
                            setScope(nextScope);
                            window.scrollTo({
                                top: 0,
                                left: 0,
                                behavior: 'auto',
                            });
                        }}
                    />
                }
            />

            <PageContent className="space-y-6 md:space-y-8">
                {!statsIndexQuery.isError &&
                    statsIndexTransition.showBlockingSpinner && (
                        <section className="page-centered-state">
                            <LoadingSpinner
                                size="lg"
                                srLabel="Loading statistics"
                            />
                        </section>
                    )}

                {statsIndexQuery.isError && (
                    <PageErrorState
                        error={statsIndexQuery.error}
                        onRetry={() => statsIndexQuery.refetch()}
                    />
                )}

                {statsIndex && (
                    <div className="relative space-y-6 md:space-y-8">
                        {statsIndexTransition.showOverlaySpinner && (
                            <div className="absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                <LoadingSpinner
                                    size="md"
                                    srLabel="Loading statistics"
                                />
                            </div>
                        )}

                        {showPageEmptyState ? (
                            <StatisticsEmptyState />
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
                                    yearData={
                                        heatmapYearTransition.displayData ??
                                        undefined
                                    }
                                    loading={heatmapLoading}
                                    animationSeed={location.key}
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
                    </div>
                )}
            </PageContent>
        </>
    );
}
