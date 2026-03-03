import { useEffect, useMemo, useRef } from 'react';
import { LuBookOpen, LuCalendarDays, LuClock3 } from 'react-icons/lu';

import {
    scrollToHorizontalOverflowRatio,
    scrollToHorizontalPosition,
} from '../../../shared/lib/dom/horizontal-scroll';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { DataFormatter } from '../lib/formatters';
import { monthKeyAt, toShortMonthKey } from '../lib/months';
import { translation } from '../../../shared/i18n';
import { TooltipManager } from '../../../shared/overlay/tooltip-manager';
import {
    type MonthlyReadStats,
    type SectionName,
    type YearlySummaryStats,
} from '../model/statistics-model';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { YearSelector } from '../../../shared/ui/selectors/YearSelector';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

type YearlyStatsSectionProps = {
    visible: boolean;
    onToggle: (sectionName: SectionName) => void;
    availableYears: number[];
    selectedYear: number | null;
    onSelectYear: (year: number) => void;
    yearlySummary: YearlySummaryStats;
    yearlyMonthlyStats: MonthlyReadStats[];
    isFetching: boolean;
};

export function YearlyStatsSection({
    visible,
    onToggle,
    availableYears,
    selectedYear,
    onSelectYear,
    yearlySummary,
    yearlyMonthlyStats,
    isFetching,
}: YearlyStatsSectionProps) {
    const scrollContainerRef = useRef<HTMLDivElement>(null);
    const chartContentRef = useRef<HTMLDivElement>(null);

    const maxReadTime = useMemo(
        () => Math.max(...yearlyMonthlyStats.map((month) => month.read_time), 0),
        [yearlyMonthlyStats],
    );

    useEffect(() => {
        const scrollContainer = scrollContainerRef.current;
        const chartContent = chartContentRef.current;
        if (!scrollContainer || !chartContent || !selectedYear) {
            return;
        }

        requestAnimationFrame(() => {
            if (selectedYear === new Date().getFullYear()) {
                const monthWidth = chartContent.scrollWidth / 12;
                const targetPosition = new Date().getMonth() * monthWidth;
                scrollToHorizontalPosition(scrollContainer, chartContent, targetPosition, 0.7);
                return;
            }

            scrollToHorizontalOverflowRatio(scrollContainer, chartContent, 0.8);
        });
    }, [selectedYear, yearlyMonthlyStats]);

    return (
        <CollapsibleSection
            sectionKey="yearly-stats"
            accentClass="bg-gradient-to-b from-violet-400 to-violet-600"
            title={translation.get('yearly-statistics')}
            visible={visible}
            onToggle={() => onToggle('yearly-stats')}
            controls={
                <YearSelector
                    idPrefix="YearlyStatsYear"
                    years={availableYears}
                    selectedYear={selectedYear}
                    onSelect={onSelectYear}
                    iconColorClass="text-gray-600 dark:text-gray-300 sm:text-violet-400 sm:dark:text-violet-400"
                    optionActiveClass="bg-green-50 dark:bg-dark-700 text-green-900 dark:text-white"
                    mobileFallback="--"
                />
            }
        >
            <div className="mb-4 sm:mb-5 grid grid-cols-1 sm:grid-cols-3 gap-3 sm:gap-4">
                <MetricCard
                    variant="compact"
                    icon={LuClock3}
                    iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                    iconClassName="text-primary-600 dark:text-white"
                    valueId="yearlyStatsReadTime"
                    value={DataFormatter.formatReadTimeWithDays(yearlySummary.read_time)}
                    label={translation.get('total-read-time')}
                />

                <MetricCard
                    variant="compact"
                    icon={LuBookOpen}
                    iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                    iconClassName="text-indigo-600 dark:text-white"
                    valueId="yearlyStatsCompletedCount"
                    value={DataFormatter.formatCount(yearlySummary.completed_count)}
                    label={translation.get('completed-books')}
                />

                <MetricCard
                    variant="compact"
                    icon={LuCalendarDays}
                    iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                    iconClassName="text-green-600 dark:text-white"
                    valueId="yearlyStatsActiveDays"
                    value={DataFormatter.formatCount(yearlySummary.active_days)}
                    label={translation.get('active-days', yearlySummary.active_days)}
                />
            </div>

            <div className="mb-8">
                <div className="relative bg-white dark:bg-dark-850/50 rounded-lg p-3 sm:p-4 md:p-5 border border-gray-200/30 dark:border-dark-700/70 overflow-hidden">
                    <div
                        id="yearlyStatsLoadingIndicator"
                        className={`absolute inset-0 bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px] z-10 flex items-center justify-center ${isFetching ? '' : 'hidden'}`}
                    >
                        <LoadingSpinner size="md" srLabel="Loading yearly statistics" />
                    </div>

                    <div
                        id="yearlyStatsEmptyState"
                        className={`${availableYears.length === 0 ? '' : 'hidden'} rounded-lg border border-dashed border-gray-300/80 dark:border-dark-700 p-8 text-center text-sm text-gray-500 dark:text-dark-300`}
                    >
                        {translation.get('stats-empty.nothing-here')}
                    </div>

                    <div
                        id="yearlyStatsChart"
                        className={`transition-opacity duration-300 ${isFetching ? 'opacity-50' : ''}`}
                    >
                        <div
                            id="yearlyStatsScrollContainer"
                            className="overflow-x-auto overflow-y-hidden scrollbar-hide"
                            ref={scrollContainerRef}
                        >
                            <div
                                id="yearlyStatsChartContent"
                                className="min-w-[680px] lg:min-w-0"
                                ref={chartContentRef}
                            >
                                <div
                                    id="yearlyStatsBars"
                                    className="h-56 sm:h-64 lg:h-72 grid grid-cols-12 gap-2 sm:gap-3 items-end"
                                >
                                    {Array.from({ length: 12 }, (_, monthIndex) => {
                                        const stats = yearlyMonthlyStats[monthIndex] ?? {
                                            read_time: 0,
                                            pages_read: 0,
                                            active_days: 0,
                                        };

                                        let heightPercent = 2;
                                        if (maxReadTime > 0 && stats.read_time > 0) {
                                            heightPercent = Math.max(
                                                (stats.read_time / maxReadTime) * 100,
                                                8,
                                            );
                                        }

                                        const monthLabel = translation.get(monthKeyAt(monthIndex));
                                        const valueLabel = DataFormatter.formatReadTime(
                                            stats.read_time,
                                        );
                                        const pagesLabel = translation.get(
                                            'pages',
                                            stats.pages_read,
                                        );
                                        const activeDaysLabel = translation.get(
                                            'active-days-tooltip',
                                            stats.active_days,
                                        );
                                        const formattedActiveDays = DataFormatter.formatCount(
                                            stats.active_days,
                                        );
                                        const tooltip = selectedYear
                                            ? `${monthLabel} ${selectedYear}: ${valueLabel}, ${pagesLabel}, ${formattedActiveDays} ${activeDaysLabel}`
                                            : `${monthLabel}: ${valueLabel}`;

                                        return (
                                            <div
                                                key={monthIndex}
                                                className="h-full flex flex-col justify-end"
                                            >
                                                <div className="relative h-full flex items-end">
                                                    <div
                                                        className="yearly-stat-bar-fill cursor-pointer w-full rounded-t-sm bg-gradient-to-t from-indigo-600 to-violet-500 shadow-[0_-2px_16px_rgba(99,102,241,0.35)] opacity-35 transition-[height,opacity] duration-500 ease-out overflow-hidden"
                                                        style={{
                                                            height: `${heightPercent}%`,
                                                            opacity: stats.read_time > 0 ? 1 : 0.35,
                                                        }}
                                                        data-tooltip-gap="5"
                                                        aria-label={tooltip}
                                                        ref={(element) => {
                                                            if (element) {
                                                                TooltipManager.attach(
                                                                    element,
                                                                    tooltip,
                                                                );
                                                            }
                                                        }}
                                                    >
                                                        <span className="block h-[2px] w-full bg-white/75 dark:bg-white/45"></span>
                                                    </div>
                                                </div>
                                                <div className="mt-3 text-center text-xs text-gray-500 dark:text-dark-400 leading-none">
                                                    {translation.get(
                                                        toShortMonthKey(monthKeyAt(monthIndex)),
                                                    )}
                                                </div>
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </CollapsibleSection>
    );
}
