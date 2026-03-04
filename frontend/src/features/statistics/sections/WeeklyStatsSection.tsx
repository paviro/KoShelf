import { LuClock3, LuFileText } from 'react-icons/lu';

import type { StatisticsIndexWeek, StatisticsWeekResponse } from '../api/statistics-data';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { DataFormatter } from '../lib/formatters';
import { translation } from '../../../shared/i18n';
import { formatSessionDuration, type SectionName } from '../model/statistics-model';
import { WeekSelector } from '../components/WeekSelector';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

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
    return (
        <CollapsibleSection
            sectionKey="weekly-stats"
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
                    className={`absolute inset-0 bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px] z-10 flex items-center justify-center rounded-xl ${loading ? '' : 'hidden'}`}
                >
                    <LoadingSpinner
                        size="md"
                        srLabel="Loading weekly statistics"
                        spinnerClassName="border-primary-400/35 dark:border-primary-900 border-t-primary-300 dark:border-t-primary-300"
                    />
                </div>

                <div
                    className="week-stats grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3"
                >
                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                        iconClassName="text-primary-600 dark:text-white"
                        valueId="weekReadTime"
                        value={DataFormatter.formatReadTime(weeklyStats.read_time)}
                        label={translation.get('total-read-time')}
                    />

                    <MetricCard
                        icon={LuFileText}
                        iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                        iconClassName="text-indigo-600 dark:text-white"
                        valueId="weekPagesRead"
                        value={DataFormatter.formatCount(weeklyStats.pages_read)}
                        label={translation.get('total-pages-read')}
                    />

                    <MetricCard
                        icon={LuFileText}
                        iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                        iconClassName="text-amber-600 dark:text-white"
                        valueId="weekAvgPagesPerDay"
                        value={DataFormatter.formatAvgPages(weeklyStats.avg_pages_per_day)}
                        label={translation.get('average-pages-day')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                        iconClassName="text-green-600 dark:text-white"
                        valueId="weekAvgReadTimePerDay"
                        value={DataFormatter.formatMinutes(weeklyStats.avg_read_time_per_day / 60)}
                        label={translation.get('average-time-day')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                        iconClassName="text-pink-600 dark:text-white"
                        valueId="weekLongestSession"
                        value={formatSessionDuration(weeklyStats.longest_session_duration)}
                        label={translation.get('session.longest')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                        iconClassName="text-purple-600 dark:text-white"
                        valueId="weekAverageSession"
                        value={formatSessionDuration(weeklyStats.average_session_duration)}
                        label={translation.get('session.average')}
                    />
                </div>
            </div>
        </CollapsibleSection>
    );
}
