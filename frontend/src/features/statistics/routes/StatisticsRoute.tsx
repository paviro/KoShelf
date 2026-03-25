import { useEffect, useMemo, useState } from 'react';
import { useLocation } from 'react-router';

import { useDocumentTitle } from '../../../shared/hooks/useDocumentTitle';
import { useSiteQuery } from '../../../shared/hooks/useSiteQuery';
import { translation } from '../../../shared/i18n';
import { useSectionVisibilityState } from '../../../shared/lib/state/useSectionVisibilityState';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import { QueryStateLayout } from '../../../shared/ui/feedback/QueryStateLayout';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';
import type { StatisticsWeekResponse } from '../api/statistics-data';
import { ScopeFilter } from '../components/ScopeFilter';
import {
    useStatisticsIndexQuery,
    useStatisticsWeekQuery,
    useStatisticsYearQuery,
    useStatisticsYearlySectionQuery,
} from '../hooks/useStatisticsQueries';
import {
    SECTION_NAMES,
    defaultSectionState,
    isCurrentStreakActive,
    persistStatisticsViewState,
    readStoredStatisticsViewState,
    type MonthlyReadStats,
    type YearlySummaryStats,
    type SectionName,
} from '../model/statistics-model';
import { OverallStatsSection } from '../sections/OverallStatsSection';
import { ReadingStreakSection } from '../sections/ReadingStreakSection';
import { StatisticsEmptyState } from '../sections/StatisticsEmptyState';
import { WeeklyStatsSection } from '../sections/WeeklyStatsSection';
import { YearlyStatsSection } from '../sections/YearlyStatsSection';

const emptyMonthlyStats: MonthlyReadStats[] = Array.from(
    { length: 12 },
    () => ({
        reading_time_sec: 0,
        pages_read: 0,
        active_days: 0,
    }),
);

const emptySummary: YearlySummaryStats = {
    reading_time_sec: 0,
    completions: 0,
    active_days: 0,
};

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

    const { siteQuery, showTypeFilter } = useSiteQuery();

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
        () => [...(statsIndex?.available_years ?? [])].reverse(),
        [statsIndex?.available_years],
    );
    const availableWeeks = useMemo(
        () => [...(statsIndex?.available_weeks ?? [])].reverse(),
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

    useDocumentTitle(
        translation.get('reading-statistics'),
        siteQuery.data?.title,
    );

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
    const yearlySectionQuery = useStatisticsYearlySectionQuery(
        scope,
        effectiveSelectedYearlyYear,
    );
    const yearlySectionTransition = useQueryTransitionState({
        data: yearlySectionQuery.data,
        enabled: Boolean(effectiveSelectedYearlyYear),
        isLoading: yearlySectionQuery.isLoading,
        isFetching: yearlySectionQuery.isFetching,
        isPlaceholderData: yearlySectionQuery.isPlaceholderData,
    });
    const effectiveDisplayedYearlySectionData =
        yearlySectionTransition.displayData;

    const weeklyLoading =
        weekTransition.showBlockingSpinner || weekTransition.showOverlaySpinner;
    const yearlyLoading =
        yearlySectionTransition.showBlockingSpinner ||
        yearlySectionTransition.showOverlaySpinner;
    const heatmapLoading =
        heatmapYearTransition.showBlockingSpinner ||
        heatmapYearTransition.showOverlaySpinner;

    const yearlyMonthlyStats =
        effectiveDisplayedYearlySectionData?.monthlyStats ?? emptyMonthlyStats;
    const yearlySummary =
        effectiveDisplayedYearlySectionData?.yearlySummary ?? emptySummary;

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
                <QueryStateLayout
                    isError={statsIndexQuery.isError}
                    error={statsIndexQuery.error}
                    onRetry={() => statsIndexQuery.refetch()}
                    showBlockingSpinner={
                        statsIndexTransition.showBlockingSpinner
                    }
                    showOverlaySpinner={statsIndexTransition.showOverlaySpinner}
                    hasData={Boolean(statsIndex)}
                    srLabel="Loading statistics"
                    renderContent={() =>
                        showPageEmptyState ? (
                            <StatisticsEmptyState />
                        ) : (
                            <>
                                <OverallStatsSection
                                    visible={sectionState['overall-stats']}
                                    onToggle={toggleSection}
                                    overview={statsIndex!.overview}
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
                                    longestStreak={statsIndex!.streaks.longest}
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
                        )
                    }
                />
            </PageContent>
        </>
    );
}
