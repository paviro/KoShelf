import { LuBookOpen, LuCalendarDays, LuClock3, LuFileText } from 'react-icons/lu';

import type { ScopeValue } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import type { CalendarMonthlyStats } from '../api/calendar-data';
import { formatDuration } from '../model/calendar-model';

type CalendarMonthlyStatsSectionProps = {
    stats: CalendarMonthlyStats;
    scope: ScopeValue;
};

export function CalendarMonthlyStatsSection({ stats, scope }: CalendarMonthlyStatsSectionProps) {
    const completedLabel =
        scope === 'comics'
            ? translation.get('comic-label', { count: stats.books_read })
            : translation.get('book-label', { count: stats.books_read });

    return (
        <section className="grid grid-cols-2 xl:grid-cols-4 gap-3 sm:gap-4">
            <MetricCard
                icon={LuBookOpen}
                iconContainerClassName="bg-blue-500/20 dark:bg-gradient-to-br dark:from-blue-500 dark:to-blue-600"
                iconClassName="text-blue-600 dark:text-white"
                value={stats.books_read}
                label={completedLabel}
            />
            <MetricCard
                icon={LuFileText}
                iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                iconClassName="text-green-600 dark:text-white"
                value={formatNumber(stats.pages_read)}
                label={translation.get('total-pages-read')}
            />
            <MetricCard
                icon={LuClock3}
                iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                iconClassName="text-purple-600 dark:text-white"
                value={formatDuration(stats.time_read)}
                label={translation.get('total-read-time')}
            />
            <MetricCard
                icon={LuCalendarDays}
                iconContainerClassName="bg-orange-500/20 dark:bg-gradient-to-br dark:from-orange-500 dark:to-orange-600"
                iconClassName="text-orange-600 dark:text-white"
                value={`${stats.days_read_pct}%`}
                label={translation.get('active-days', stats.days_read_pct)}
            />
        </section>
    );
}
