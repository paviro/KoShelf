import { useMemo } from 'react';
import { LuClock3, LuFileText } from 'react-icons/lu';

import type {
    DailyActivityEntry,
    StatisticsIndexWeek,
    StatisticsWeekResponse,
} from '../api/statistics-data';
import { parsePlainDate } from '../../../shared/lib/intl/formatDate';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { DataFormatter } from '../lib/formatters';
import { translation } from '../../../shared/i18n';
import {
    formatSessionDurationParts,
    type SectionName,
} from '../model/statistics-model';
import { WeekSelector } from '../components/WeekSelector';
import {
    DistributionBarChart,
    type DistributionBarItem,
} from '../components/DistributionBarChart';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { MetricCardUnitValue } from '../../../shared/ui/cards/MetricCardUnitValue';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

const WEEKDAY_TRANSLATION_KEYS = [
    'weekday.mon',
    'weekday.tue',
    'weekday.wed',
    'weekday.thu',
    'weekday.fri',
    'weekday.sat',
    'weekday.sun',
] as const;

const MILLIS_PER_DAY = 1000 * 60 * 60 * 24;

function buildWeekdayBarItems(
    dailyActivity: DailyActivityEntry[],
    startDate: string,
): DistributionBarItem[] {
    const days = WEEKDAY_TRANSLATION_KEYS.map((key) => ({
        read_time: 0,
        pages_read: 0,
        label: translation.get(key),
    }));

    if (startDate) {
        const start = parsePlainDate(startDate);
        if (!start) {
            return days.map((day) => ({
                readTime: day.read_time,
                tooltip: `${day.label}: ${DataFormatter.formatReadTime(day.read_time)}, ${translation.get('pages', day.pages_read)}`,
                label: day.label,
            }));
        }

        for (const entry of dailyActivity) {
            const current = parsePlainDate(entry.date);
            if (!current) {
                continue;
            }
            const diffDays = Math.round(
                (current.getTime() - start.getTime()) / (1000 * 60 * 60 * 24),
            );
            if (diffDays >= 0 && diffDays < 7) {
                days[diffDays].read_time += entry.read_time;
                days[diffDays].pages_read += entry.pages_read;
            }
        }
    }

    return days.map((day) => ({
        readTime: day.read_time,
        tooltip: `${day.label}: ${DataFormatter.formatReadTime(day.read_time)}, ${translation.get('pages', day.pages_read)}`,
        label: day.label,
    }));
}

function weekAverageDayCount(startDate: string, endDate: string): number {
    const start = parsePlainDate(startDate);
    const end = parsePlainDate(endDate);
    if (!start || !end) {
        return 7;
    }

    const now = new Date();
    const today = new Date(
        Date.UTC(now.getFullYear(), now.getMonth(), now.getDate(), 12, 0, 0, 0),
    );

    if (today < start || today > end) {
        return 7;
    }

    const elapsedDays =
        Math.floor((today.getTime() - start.getTime()) / MILLIS_PER_DAY) + 1;
    return Math.min(Math.max(elapsedDays, 1), 7);
}

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
    const weekdayBarItems = useMemo(
        () =>
            buildWeekdayBarItems(
                weeklyStats.daily_activity,
                weeklyStats.start_date,
            ),
        [weeklyStats.daily_activity, weeklyStats.start_date],
    );
    const averageDayCount = weekAverageDayCount(
        weeklyStats.start_date,
        weeklyStats.end_date,
    );
    const averagePagesPerDay = weeklyStats.pages_read / averageDayCount;
    const averageReadTimePerDay = weeklyStats.read_time / averageDayCount;

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

                <div className="week-stats grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3">
                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                        iconClassName="text-primary-600 dark:text-white"
                        valueId="weekReadTime"
                        value={
                            <MetricCardUnitValue
                                value={DataFormatter.formatReadTimeParts(
                                    weeklyStats.read_time,
                                )}
                            />
                        }
                        label={translation.get('total-read-time')}
                    />

                    <MetricCard
                        icon={LuFileText}
                        iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                        iconClassName="text-indigo-600 dark:text-white"
                        valueId="weekPagesRead"
                        value={DataFormatter.formatCount(
                            weeklyStats.pages_read,
                        )}
                        label={translation.get('total-pages-read')}
                    />

                    <MetricCard
                        icon={LuFileText}
                        iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                        iconClassName="text-amber-600 dark:text-white"
                        valueId="weekAvgPagesPerDay"
                        value={DataFormatter.formatAvgPages(averagePagesPerDay)}
                        label={translation.get('average-pages-day')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                        iconClassName="text-green-600 dark:text-white"
                        valueId="weekAvgReadTimePerDay"
                        value={
                            <MetricCardUnitValue
                                value={DataFormatter.formatMinutesParts(
                                    averageReadTimePerDay / 60,
                                )}
                            />
                        }
                        label={translation.get('average-time-day')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                        iconClassName="text-pink-600 dark:text-white"
                        valueId="weekLongestSession"
                        value={
                            <MetricCardUnitValue
                                value={formatSessionDurationParts(
                                    weeklyStats.longest_session_duration,
                                )}
                            />
                        }
                        label={translation.get('session.longest')}
                    />

                    <MetricCard
                        icon={LuClock3}
                        iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                        iconClassName="text-purple-600 dark:text-white"
                        valueId="weekAverageSession"
                        value={
                            <MetricCardUnitValue
                                value={formatSessionDurationParts(
                                    weeklyStats.average_session_duration,
                                )}
                            />
                        }
                        label={translation.get('session.average')}
                    />
                </div>

                <div className="mt-4 sm:mt-5 bg-white dark:bg-dark-850/50 rounded-lg p-3 sm:p-4 md:p-5 border border-gray-200/30 dark:border-dark-700/70">
                    <DistributionBarChart
                        items={weekdayBarItems}
                        columns={7}
                        heightClassName="h-44 sm:h-52 lg:h-56"
                        barClassName="from-blue-600 to-sky-400 shadow-[0_-2px_16px_rgba(56,189,248,0.3)]"
                    />
                </div>
            </div>
        </CollapsibleSection>
    );
}
