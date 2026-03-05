import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import { useLocation } from 'react-router-dom';

import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';
import { OverallStatsSection } from '../sections/OverallStatsSection';
import { ReadingStreakSection } from '../sections/ReadingStreakSection';
import { ScopeFilter } from '../components/ScopeFilter';
import { WeeklyStatsSection } from '../sections/WeeklyStatsSection';
import { YearlyStatsSection } from '../sections/YearlyStatsSection';
import { useSectionVisibilityState } from '../../../shared/lib/state/useSectionVisibilityState';
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
import type { SiteResponse } from '../../../shared/contracts';
import type { StatisticsWeekResponse } from '../api/statistics-data';
import { translation } from '../../../shared/i18n';

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

export function StatisticsRoute() {
    const location = useLocation();
    const [initialViewState] = useState(() => readStoredStatisticsViewState());
    const [scope, setScope] = useState(() => initialViewState.scope);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const statsIndexQuery = useStatisticsIndexQuery(scope);
    const sectionDefaults = useMemo(() => defaultSectionState(), []);
    const { state: sectionState, toggle: toggleSection } =
        useSectionVisibilityState<SectionName>({
            routeId: 'statistics',
            sectionKeys: SECTION_NAMES,
            defaults: sectionDefaults,
        });

    const availableYears = useMemo(
        () => statsIndexQuery.data?.available_years ?? [],
        [statsIndexQuery.data?.available_years],
    );
    const availableWeeks = useMemo(
        () => statsIndexQuery.data?.available_weeks ?? [],
        [statsIndexQuery.data?.available_weeks],
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

    const effectiveSelectedWeekKey = useMemo(() => {
        if (
            selectedWeekKey &&
            availableWeeks.some((week) => week.week_key === selectedWeekKey)
        ) {
            return selectedWeekKey;
        }

        return availableWeeks[0]?.week_key ?? null;
    }, [availableWeeks, selectedWeekKey]);
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
        if (!statsIndexQuery.isSuccess) {
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
        statsIndexQuery.isSuccess,
    ]);

    const weekQuery = useStatisticsWeekQuery(scope, effectiveSelectedWeekKey);
    const heatmapYearQuery = useStatisticsYearQuery(
        scope,
        effectiveSelectedHeatmapYear,
    );
    const yearlyQuery = useStatisticsYearQuery(
        scope,
        effectiveSelectedYearlyYear,
    );
    const effectiveDisplayedYearlyData = yearlyQuery.data ?? null;

    const statsIndex = statsIndexQuery.data;
    const weeklyLoading = weekQuery.isFetching;

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
                effectiveDisplayedYearlyData?.summary.completed_count ?? 0,
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
        Boolean(statsIndex) &&
        availableYears.length === 0 &&
        availableWeeks.length === 0;
    const yearlyLoading = yearlyQuery.isFetching;

    const weeklyStats = weekQuery.data ?? EMPTY_WEEKLY_STATS;

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
                {statsIndexQuery.isLoading && (
                    <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                        <LoadingSpinner
                            size="lg"
                            srLabel="Loading statistics"
                        />
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
                                    {translation.get(
                                        'stats-empty.nothing-here',
                                    )}
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
                    </>
                )}
            </PageContent>
        </>
    );
}
