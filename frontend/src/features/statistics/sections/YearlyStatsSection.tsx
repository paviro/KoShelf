import { useEffect, useMemo, useRef } from 'react';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuCalendarDays, LuClock3 } from 'react-icons/lu';

import {
    scrollToHorizontalOverflowRatio,
    scrollToHorizontalPosition,
} from '../../../shared/lib/dom/horizontal-scroll';
import {
    formatMonthKey,
    formatMonthOfYear,
} from '../../../shared/lib/intl/formatDate';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { DataFormatter } from '../lib/formatters';
import { translation } from '../../../shared/i18n';
import {
    type MonthlyReadStats,
    type SectionName,
    type YearlySummaryStats,
} from '../model/statistics-model';
import {
    DistributionBarChart,
    type DistributionBarItem,
} from '../components/DistributionBarChart';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { MetricCardUnitValue } from '../../../shared/ui/cards/MetricCardUnitValue';
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

    const monthlyBarItems: DistributionBarItem[] = useMemo(
        () =>
            Array.from({ length: 12 }, (_, monthIndex) => {
                const stats = yearlyMonthlyStats[monthIndex] ?? {
                    reading_time_sec: 0,
                    pages_read: 0,
                    active_days: 0,
                };

                const monthKey = `${selectedYear ?? 2026}-${String(
                    monthIndex + 1,
                ).padStart(2, '0')}`;
                const monthLabel = formatMonthKey(monthKey, {
                    monthStyle: 'long',
                    includeYear: Boolean(selectedYear),
                });
                const valueLabel = DataFormatter.formatReadTime(
                    stats.reading_time_sec,
                );
                const pagesLabel = translation.get('pages', stats.pages_read);
                const activeDaysLabel = translation.get(
                    'active-days-tooltip',
                    stats.active_days,
                );
                const formattedActiveDays = DataFormatter.formatCount(
                    stats.active_days,
                );

                return {
                    readTime: stats.reading_time_sec,
                    tooltip: `${monthLabel}: ${valueLabel}, ${pagesLabel}, ${formattedActiveDays} ${activeDaysLabel}`,
                    label: formatMonthOfYear(monthIndex, {
                        monthStyle: 'short',
                    }),
                };
            }),
        [yearlyMonthlyStats, selectedYear],
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
                scrollToHorizontalPosition(
                    scrollContainer,
                    chartContent,
                    targetPosition,
                    0.7,
                );
                return;
            }

            scrollToHorizontalOverflowRatio(scrollContainer, chartContent, 0.8);
        });
    }, [selectedYear, yearlyMonthlyStats]);

    return (
        <CollapsibleSection
            sectionKey="yearly-stats"
            accentClass="bg-linear-to-b from-violet-400 to-violet-600"
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
                    optionActiveClass="bg-green-50/50 dark:bg-dark-700/50 text-green-900 dark:text-white"
                    mobileFallback="--"
                />
            }
        >
            <div className="mb-4 sm:mb-5 grid grid-cols-1 sm:grid-cols-3 gap-3 sm:gap-4">
                <MetricCard
                    variant="inline"
                    icon={LuClock3}
                    iconContainerClassName="bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600"
                    iconClassName="text-primary-600 dark:text-white"
                    valueId="yearlyStatsReadTime"
                    value={
                        <MetricCardUnitValue
                            value={DataFormatter.formatReadTimeWithDaysParts(
                                yearlySummary.reading_time_sec,
                            )}
                        />
                    }
                    label={translation.get('total-read-time')}
                />

                <MetricCard
                    variant="inline"
                    icon={HiOutlineBookOpen}
                    iconContainerClassName="bg-indigo-500/20 dark:bg-linear-to-br dark:from-indigo-500 dark:to-indigo-600"
                    iconClassName="text-indigo-600 dark:text-white"
                    valueId="yearlyStatsCompletedCount"
                    value={DataFormatter.formatCount(yearlySummary.completions)}
                    label={translation.get('completed-books')}
                />

                <MetricCard
                    variant="inline"
                    icon={LuCalendarDays}
                    iconContainerClassName="bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600"
                    iconClassName="text-green-600 dark:text-white"
                    valueId="yearlyStatsActiveDays"
                    value={DataFormatter.formatCount(yearlySummary.active_days)}
                    label={translation.get(
                        'active-days',
                        yearlySummary.active_days,
                    )}
                />
            </div>

            <div className="mb-8">
                <div className="relative bg-white dark:bg-dark-850/50 rounded-lg p-3 sm:p-4 md:p-5 border border-gray-200/30 dark:border-dark-700/70 overflow-hidden">
                    <div
                        id="yearlyStatsLoadingIndicator"
                        className={`absolute inset-0 bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px] z-10 flex items-center justify-center ${isFetching ? '' : 'hidden'}`}
                    >
                        <LoadingSpinner
                            size="md"
                            srLabel="Loading yearly statistics"
                        />
                    </div>

                    <div
                        id="yearlyStatsEmptyState"
                        className={`${availableYears.length === 0 ? '' : 'hidden'} rounded-lg border border-dashed border-gray-300/80 dark:border-dark-700 p-8 text-center text-sm font-medium text-gray-500 dark:text-dark-300`}
                    >
                        {translation.get('stats-empty.nothing-here')}
                    </div>

                    <div id="yearlyStatsChart">
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
                                <DistributionBarChart
                                    items={monthlyBarItems}
                                    columns={12}
                                    heightClassName="h-56 sm:h-64 lg:h-72"
                                    barClassName="from-indigo-600 to-violet-500 shadow-[0_-2px_16px_rgba(99,102,241,0.35)]"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </CollapsibleSection>
    );
}
