import { TooltipManager } from './tooltip-manager.js';
import { translation } from '../shared/i18n.js';
import { DataFormatter } from '../shared/statistics-formatters.js';
import {
    STATISTICS_MONTH_KEYS,
    monthKeyAt,
    toShortMonthKey,
    type StatisticsMonthKey,
} from '../shared/statistics-months.js';
import { loadYearlyActivity, type DailyActivityEntry } from '../shared/statistics-data-loader.js';

interface MonthlyReadStats {
    read_time: number;
    pages_read: number;
    active_days: number;
}

interface YearlySummaryStats {
    read_time: number;
    completed_count: number;
    active_days: number;
}

export class YearlyStatsChart {
    private statsJsonBasePath = '/assets/json/statistics';
    private requestId = 0;

    init(statsJsonBasePath: string): void {
        this.statsJsonBasePath = statsJsonBasePath;

        const barsContainer = document.getElementById('yearlyStatsBars');
        if (!barsContainer) return;

        this.ensureYearlyChartBars();

        const yearSelectorWrapper = document.getElementById('yearlyStatsYearSelectorWrapper');
        const yearOptions = document.getElementById('yearlyStatsYearOptions');
        const yearOptionElements = document.querySelectorAll<HTMLElement>(
            '.yearly-stats-year-option',
        );
        const emptyState = document.getElementById('yearlyStatsEmptyState');

        if (yearOptionElements.length === 0) {
            emptyState?.classList.remove('hidden');
            this.updateYearlyChart(this.createEmptyMonthlyStats(), null);
            this.updateYearlySummaryCards(this.createEmptyYearlySummary());
            return;
        }

        emptyState?.classList.add('hidden');

        if (yearSelectorWrapper && yearOptions) {
            yearSelectorWrapper.addEventListener('click', () => {
                yearOptions.classList.toggle('hidden');
            });
        }

        yearOptionElements.forEach((option) => {
            option.addEventListener('click', () => {
                const selectedYear = Number.parseInt(option.getAttribute('data-year') || '', 10);
                if (Number.isNaN(selectedYear) || selectedYear <= 0) return;

                this.updateSelectedYearlyStatsText(selectedYear);
                this.updateActiveYearOption(yearOptionElements, option);
                void this.loadYearlyData(selectedYear);

                yearOptions?.classList.add('hidden');
            });
        });

        const defaultYear = Number.parseInt(
            yearOptionElements[0].getAttribute('data-year') || '',
            10,
        );
        if (Number.isNaN(defaultYear) || defaultYear <= 0) {
            this.updateYearlyChart(this.createEmptyMonthlyStats(), null);
            this.updateYearlySummaryCards(this.createEmptyYearlySummary());
            return;
        }

        this.updateSelectedYearlyStatsText(defaultYear);
        this.updateActiveYearOption(yearOptionElements, yearOptionElements[0]);
        void this.loadYearlyData(defaultYear);
    }

    private ensureYearlyChartBars(): void {
        const barsContainer = document.getElementById('yearlyStatsBars');
        if (!barsContainer || barsContainer.childElementCount > 0) return;

        STATISTICS_MONTH_KEYS.forEach((monthKey, monthIndex) => {
            barsContainer.appendChild(this.createYearlyChartColumn(monthKey, monthIndex));
        });
    }

    private createYearlyChartColumn(monthKey: StatisticsMonthKey, monthIndex: number): HTMLElement {
        const monthColumn = document.createElement('div');
        monthColumn.className = 'h-full flex flex-col justify-end';

        const barFrame = document.createElement('div');
        barFrame.className = 'relative h-full flex items-end';

        const bar = document.createElement('div');
        bar.className =
            'yearly-stat-bar-fill w-full rounded-t-sm bg-gradient-to-t from-indigo-600 to-violet-500 shadow-[0_-2px_16px_rgba(99,102,241,0.35)] opacity-35 transition-[height,opacity] duration-500 ease-out overflow-hidden';
        bar.style.height = '2%';
        bar.setAttribute('data-month-index', String(monthIndex));
        bar.setAttribute('data-tooltip-gap', '5');

        const barCap = document.createElement('span');
        barCap.className = 'block h-[2px] w-full bg-white/75 dark:bg-white/45';
        bar.appendChild(barCap);

        barFrame.appendChild(bar);

        const monthLabel = document.createElement('div');
        monthLabel.className =
            'mt-3 text-center text-xs text-gray-500 dark:text-dark-400 leading-none';
        monthLabel.textContent = translation.get(toShortMonthKey(monthKey));

        monthColumn.appendChild(barFrame);
        monthColumn.appendChild(monthLabel);

        return monthColumn;
    }

    private updateSelectedYearlyStatsText(year: number): void {
        const selectedYearText = document.getElementById('selectedYearlyStatsText');
        if (selectedYearText) {
            selectedYearText.innerHTML = `<span class="font-bold">${year}</span>`;
        }
    }

    private updateActiveYearOption(
        allOptions: NodeListOf<HTMLElement>,
        selectedOption: HTMLElement,
    ): void {
        allOptions.forEach((el) => {
            el.classList.remove(
                'bg-green-50',
                'dark:bg-dark-700',
                'text-green-900',
                'dark:text-white',
            );
            el.classList.add('text-gray-600', 'dark:text-dark-200');
        });

        selectedOption.classList.add(
            'bg-green-50',
            'dark:bg-dark-700',
            'text-green-900',
            'dark:text-white',
        );
        selectedOption.classList.remove('text-gray-600', 'dark:text-dark-200');
    }

    private setYearlyStatsLoadingState(isLoading: boolean): void {
        const yearlyChart = document.getElementById('yearlyStatsChart');
        const loadingIndicator = document.getElementById('yearlyStatsLoadingIndicator');

        if (yearlyChart) {
            yearlyChart.classList.toggle('opacity-50', isLoading);
        }
        if (loadingIndicator) {
            loadingIndicator.classList.toggle('hidden', !isLoading);
        }
    }

    private async loadYearlyData(year: number): Promise<void> {
        const currentRequestId = ++this.requestId;
        this.setYearlyStatsLoadingState(true);

        try {
            const yearlyActivity = await loadYearlyActivity(this.statsJsonBasePath, year);

            if (currentRequestId !== this.requestId) return;

            const monthlyStats = this.aggregateMonthlyStats(yearlyActivity.data);
            const yearlySummary = this.summarizeYearlyStats(
                monthlyStats,
                yearlyActivity.summary.completed_count,
            );

            this.updateYearlySummaryCards(yearlySummary);
            this.updateYearlyChart(monthlyStats, year);
        } catch (error) {
            if (currentRequestId !== this.requestId) return;

            console.error(`Error loading yearly data for ${year}:`, error);
            this.updateYearlySummaryCards(this.createEmptyYearlySummary());
            this.updateYearlyChart(this.createEmptyMonthlyStats(), year);
        } finally {
            if (currentRequestId === this.requestId) {
                this.setYearlyStatsLoadingState(false);
            }
        }
    }

    private createEmptyMonthlyStats(): MonthlyReadStats[] {
        return Array.from({ length: 12 }, () => ({
            read_time: 0,
            pages_read: 0,
            active_days: 0,
        }));
    }

    private createEmptyYearlySummary(): YearlySummaryStats {
        return {
            read_time: 0,
            completed_count: 0,
            active_days: 0,
        };
    }

    private summarizeYearlyStats(
        monthlyStats: MonthlyReadStats[],
        completedCount: number,
    ): YearlySummaryStats {
        const summary = this.createEmptyYearlySummary();

        monthlyStats.forEach((monthStats) => {
            summary.read_time += monthStats.read_time;
            summary.active_days += monthStats.active_days;
        });

        summary.completed_count = Math.max(Math.floor(completedCount), 0);

        return summary;
    }

    private updateYearlySummaryCards(summary: YearlySummaryStats): void {
        const yearlyReadTime = document.getElementById('yearlyStatsReadTime');
        const yearlyCompletedCount = document.getElementById('yearlyStatsCompletedCount');
        const yearlyActiveDays = document.getElementById('yearlyStatsActiveDays');

        if (yearlyReadTime) {
            yearlyReadTime.textContent = DataFormatter.formatReadTimeWithDays(summary.read_time);
        }

        if (yearlyCompletedCount) {
            yearlyCompletedCount.textContent = String(summary.completed_count);
        }

        if (yearlyActiveDays) {
            yearlyActiveDays.textContent = String(summary.active_days);
        }
    }

    private aggregateMonthlyStats(dailyActivity: DailyActivityEntry[]): MonthlyReadStats[] {
        const monthlyStats = this.createEmptyMonthlyStats();

        dailyActivity.forEach((entry) => {
            const month = Number.parseInt(entry.date.slice(5, 7), 10) - 1;
            if (Number.isNaN(month) || month < 0 || month > 11) return;

            monthlyStats[month].read_time += entry.read_time;
            monthlyStats[month].pages_read += entry.pages_read;

            if (entry.read_time > 0 || entry.pages_read > 0) {
                monthlyStats[month].active_days += 1;
            }
        });

        return monthlyStats;
    }

    private updateYearlyChart(monthlyStats: MonthlyReadStats[], year: number | null): void {
        const barElements = document.querySelectorAll<HTMLElement>('.yearly-stat-bar-fill');
        if (barElements.length === 0) return;

        const maxReadTime = Math.max(...monthlyStats.map((stat) => stat.read_time), 0);

        barElements.forEach((bar, index) => {
            const monthStats = monthlyStats[index] ?? {
                read_time: 0,
                pages_read: 0,
                active_days: 0,
            };
            const readTime = monthStats.read_time;
            let heightPercent = 2;

            if (maxReadTime > 0 && readTime > 0) {
                heightPercent = Math.max((readTime / maxReadTime) * 100, 8);
            }

            bar.style.height = `${heightPercent}%`;
            bar.style.opacity = readTime > 0 ? '1' : '0.35';

            const monthLabel = translation.get(monthKeyAt(index));
            const valueLabel = DataFormatter.formatReadTime(readTime);
            const pagesLabel = translation.get('pages', monthStats.pages_read);
            const activeDaysLabel = translation.get('active-days-tooltip', monthStats.active_days);
            const tooltip = year
                ? `${monthLabel} ${year}: ${valueLabel}`
                : `${monthLabel}: ${valueLabel}`;
            const tooltipWithStats = `${tooltip}, ${pagesLabel}, ${monthStats.active_days} ${activeDaysLabel}`;

            bar.removeAttribute('title');
            TooltipManager.attach(bar, tooltipWithStats);
            bar.classList.add('cursor-pointer');
            bar.setAttribute('aria-label', tooltipWithStats);
        });
    }
}
