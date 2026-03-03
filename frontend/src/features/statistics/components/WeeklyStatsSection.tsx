import { useEffect, useState } from 'react';

import type {
    StatisticsIndexWeek,
    StatisticsWeekResponse,
} from '../../../shared/statistics-data-loader';
import { LoadingSpinner } from '../../../shared/components/LoadingSpinner';
import { DataFormatter } from '../../../shared/statistics-formatters';
import { translation } from '../../../shared/i18n';
import { formatSessionDuration, type SectionName } from '../model/statistics-model';
import { StatBadgeCard } from './StatBadgeCard';
import { StatisticsSection } from './StatisticsSection';
import { WeekSelector } from './WeekSelector';

const CLOCK_ICON = 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z';
const FILE_ICON =
    'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';

type WeeklyStatsSectionProps = {
    visible: boolean;
    onToggle: (sectionName: SectionName) => void;
    availableWeeks: StatisticsIndexWeek[];
    selectedWeekKey: string | null;
    onSelectWeek: (weekKey: string) => void;
    weeklyStats: StatisticsWeekResponse;
    loading: boolean;
};

export function WeeklyStatsSection({
    visible,
    onToggle,
    availableWeeks,
    selectedWeekKey,
    onSelectWeek,
    weeklyStats,
    loading,
}: WeeklyStatsSectionProps) {
    const [loadingVisible, setLoadingVisible] = useState(false);
    const [loadingActive, setLoadingActive] = useState(false);
    const [transitionState, setTransitionState] = useState<'transition-in' | 'transition-out'>(
        'transition-in',
    );

    useEffect(() => {
        let showTimer: number | null = null;
        let hideTimer: number | null = null;
        let transitionTimer: number | null = null;

        if (loading) {
            setTransitionState('transition-out');
            setLoadingVisible(true);
            showTimer = window.setTimeout(() => {
                setLoadingActive(true);
            }, 10);
        } else {
            setLoadingActive(false);
            hideTimer = window.setTimeout(() => {
                setLoadingVisible(false);
            }, 250);
            transitionTimer = window.setTimeout(() => {
                setTransitionState('transition-in');
            }, 50);
        }

        return () => {
            if (showTimer !== null) {
                window.clearTimeout(showTimer);
            }
            if (hideTimer !== null) {
                window.clearTimeout(hideTimer);
            }
            if (transitionTimer !== null) {
                window.clearTimeout(transitionTimer);
            }
        };
    }, [loading]);

    return (
        <StatisticsSection
            sectionName="weekly-stats"
            accentClass="bg-gradient-to-b from-blue-400 to-blue-600"
            title={translation.get('weekly-statistics')}
            visible={visible}
            onToggle={() => onToggle('weekly-stats')}
            controls={
                <WeekSelector
                    weeks={availableWeeks}
                    selectedWeekKey={selectedWeekKey}
                    onSelect={onSelectWeek}
                />
            }
        >
            <div id="weekly-statsContainer" className="relative mb-8">
                <div
                    id="statsLoadingIndicator"
                    className={`absolute inset-0 bg-dark-800/80 backdrop-blur-sm flex items-center justify-center z-10 rounded-xl ${loadingVisible ? '' : 'hidden'} ${loadingActive ? 'active' : ''}`}
                >
                    <LoadingSpinner
                        size="md"
                        srLabel="Loading weekly statistics"
                        spinnerClassName="border-primary-400/35 dark:border-primary-900 border-t-primary-300 dark:border-t-primary-300"
                    />
                </div>

                <div
                    className={`week-stats grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3 ${transitionState}`}
                >
                    <StatBadgeCard
                        iconPath={CLOCK_ICON}
                        iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                        iconClassName="text-primary-600 dark:text-white"
                        valueId="weekReadTime"
                        value={DataFormatter.formatReadTime(weeklyStats.read_time)}
                        label={translation.get('total-read-time')}
                    />

                    <StatBadgeCard
                        iconPath={FILE_ICON}
                        iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                        iconClassName="text-indigo-600 dark:text-white"
                        valueId="weekPagesRead"
                        value={weeklyStats.pages_read}
                        label={translation.get('total-pages-read')}
                    />

                    <StatBadgeCard
                        iconPath={FILE_ICON}
                        iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                        iconClassName="text-amber-600 dark:text-white"
                        valueId="weekAvgPagesPerDay"
                        value={DataFormatter.formatAvgPages(weeklyStats.avg_pages_per_day)}
                        label={translation.get('average-pages-day')}
                    />

                    <StatBadgeCard
                        iconPath={CLOCK_ICON}
                        iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                        iconClassName="text-green-600 dark:text-white"
                        valueId="weekAvgReadTimePerDay"
                        value={
                            <>
                                {Math.floor(weeklyStats.avg_read_time_per_day / 60)}
                                {translation.get('units.m')}
                            </>
                        }
                        label={translation.get('average-time-day')}
                    />

                    <StatBadgeCard
                        iconPath={CLOCK_ICON}
                        iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                        iconClassName="text-pink-600 dark:text-white"
                        valueId="weekLongestSession"
                        value={formatSessionDuration(weeklyStats.longest_session_duration)}
                        label={translation.get('session.longest')}
                    />

                    <StatBadgeCard
                        iconPath={CLOCK_ICON}
                        iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                        iconClassName="text-purple-600 dark:text-white"
                        valueId="weekAverageSession"
                        value={formatSessionDuration(weeklyStats.average_session_duration)}
                        label={translation.get('session.average')}
                    />
                </div>
            </div>
        </StatisticsSection>
    );
}
