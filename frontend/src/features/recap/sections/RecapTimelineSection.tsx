import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuClock3 } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import type { CompletionGroup, RecapScope } from '../api/recap-data';
import { formatRecapDuration, formatRecapMonth } from '../lib/recap-formatters';
import { RecapItemCard } from '../components/RecapItemCard';

type RecapTimelineSectionProps = {
    months: CompletionGroup[];
    scope: RecapScope;
};

function completionLabel(scope: RecapScope, count: number): string {
    if (scope === 'books') {
        return translation.get('book-label', count);
    }
    if (scope === 'comics') {
        return translation.get('comic-label', count);
    }
    return translation.get('status.completed');
}

export function RecapTimelineSection({
    months,
    scope,
}: RecapTimelineSectionProps) {
    return (
        <>
            {months.map((month) => (
                <div
                    key={month.key}
                    className="month-group space-y-6"
                    data-month={month.key}
                >
                    <div className="relative pl-10 recap-event">
                        <span className="recap-dot bg-gray-400 dark:bg-dark-400"></span>
                        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-2">
                            <h3 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                                {formatRecapMonth(month.key)}
                            </h3>

                            <div className="flex items-center gap-2">
                                <div className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-gradient-to-br from-blue-500/10 to-blue-400/5 dark:from-blue-500/20 dark:to-blue-400/10 border border-blue-200/50 dark:border-blue-700/30 text-blue-700 dark:text-blue-300 text-sm">
                                    <HiOutlineBookOpen
                                        className="w-4 h-4"
                                        aria-hidden
                                    />
                                    <span className="month-books-finished font-semibold">
                                        {month.items_finished}
                                    </span>
                                    {completionLabel(
                                        scope,
                                        month.items_finished,
                                    )}
                                </div>

                                <div className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-gradient-to-br from-purple-500/10 to-purple-400/5 dark:from-purple-500/20 dark:to-purple-400/10 border border-purple-200/50 dark:border-purple-700/30 text-purple-700 dark:text-purple-300 text-sm">
                                    <LuClock3 className="w-4 h-4" aria-hidden />
                                    <span className="month-hours-display font-semibold">
                                        {formatRecapDuration(
                                            month.reading_time_sec,
                                        )}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    {month.items.map((item, index) => (
                        <RecapItemCard
                            key={`${month.key}:${item.title}:${item.end_date}:${index}`}
                            item={item}
                        />
                    ))}
                </div>
            ))}
        </>
    );
}
