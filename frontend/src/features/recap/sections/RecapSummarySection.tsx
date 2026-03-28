import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuClock3, LuFlame, LuSparkles, LuZap } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { CompletionsSummary, RecapScope } from '../api/recap-data';
import { formatRecapMonth } from '../lib/recap-formatters';
import { RecapActiveDaysCard } from '../components/RecapActiveDaysCard';
import { RecapDurationValue } from '../components/RecapDurationValue';
import { RecapFeaturedStatCard } from '../components/RecapFeaturedStatCard';
import { RecapStatCard } from '../components/RecapStatCard';

type RecapSummarySectionProps = {
    year: number;
    scope: RecapScope;
    summary: CompletionsSummary;
};

function isLeapYear(year: number): boolean {
    return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
}

function completionLabel(scope: RecapScope, total: number): string {
    if (scope === 'books') {
        return translation.get('books-finished', total);
    }
    if (scope === 'comics') {
        return translation.get('comics-finished', total);
    }
    return translation.get('status.completed');
}

function decomposeSeconds(totalSeconds: number): {
    days: number;
    hours: number;
    minutes: number;
} {
    const safe = Number.isFinite(totalSeconds)
        ? Math.max(0, Math.floor(totalSeconds))
        : 0;
    return {
        days: Math.floor(safe / 86400),
        hours: Math.floor((safe % 86400) / 3600),
        minutes: Math.floor((safe % 3600) / 60),
    };
}

export function RecapSummarySection({
    year,
    scope,
    summary,
}: RecapSummarySectionProps) {
    const daysInYear = isLeapYear(year) ? 366 : 365;
    const bestMonth = summary.best_month
        ? formatRecapMonth(summary.best_month)
        : null;
    const yearlySummaryLabel = translation.get('yearly-summary', {
        count: String(year),
    });

    const totalTime = decomposeSeconds(summary.total_reading_time_sec);
    const avgSession = decomposeSeconds(summary.average_session_duration_sec);
    const longestSession = decomposeSeconds(
        summary.longest_session_duration_sec,
    );

    return (
        <section className="relative pl-10 recap-event">
            <span className="recap-dot recap-dot-top bg-gray-400 dark:bg-dark-400"></span>
            <div className="flex flex-col space-y-3">
                <h3 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                    {yearlySummaryLabel}
                </h3>

                <div className="grid grid-cols-2 lg:grid-cols-6 gap-2 md:gap-3">
                    <RecapActiveDaysCard
                        activeDays={summary.active_days}
                        daysInYear={daysInYear}
                        activeDaysPercentage={summary.active_days_percentage}
                        className="col-span-2 lg:col-span-2 lg:row-span-2"
                    />

                    <RecapFeaturedStatCard
                        icon={HiOutlineBookOpen}
                        color="blue"
                        value={
                            <span className="text-2xl/none md:text-3xl font-black text-gray-900 dark:text-white">
                                {formatNumber(summary.total_items)}
                            </span>
                        }
                        label={completionLabel(scope, summary.total_items)}
                        className="col-span-1 lg:col-span-2"
                    />

                    <RecapFeaturedStatCard
                        icon={LuClock3}
                        color="purple"
                        blobPosition="top-left"
                        value={
                            <RecapDurationValue
                                hours={totalTime.hours}
                                minutes={totalTime.minutes}
                                days={totalTime.days}
                                responsiveLabels
                                valueClassName="text-2xl/none md:text-3xl font-black text-gray-900 dark:text-white"
                            />
                        }
                        label={translation.get('total-read-time')}
                        className="col-span-1 lg:col-span-2"
                    />

                    <RecapStatCard
                        icon={LuFlame}
                        color="red"
                        value={
                            <span className="text-xl font-bold text-gray-900 dark:text-white leading-none">
                                {formatNumber(summary.longest_streak_days)}{' '}
                                <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                                    {translation.get(
                                        'days_label',
                                        summary.longest_streak_days,
                                    )}
                                </span>
                            </span>
                        }
                        label={translation.get('streak.longest')}
                        className="col-span-1 lg:col-span-2 xl:col-span-1"
                    />

                    {bestMonth && (
                        <RecapStatCard
                            icon={LuSparkles}
                            color="amber"
                            value={
                                <span className="text-lg font-bold text-gray-900 dark:text-white leading-none truncate">
                                    {bestMonth}
                                </span>
                            }
                            label={translation.get('best-month')}
                            className="col-span-1 lg:col-span-2 xl:col-span-1"
                        />
                    )}

                    <RecapStatCard
                        icon={LuClock3}
                        color="orange"
                        value={
                            <RecapDurationValue
                                hours={avgSession.hours}
                                minutes={avgSession.minutes}
                            />
                        }
                        label={translation.get('session.average')}
                        className="col-span-1 lg:col-span-2 xl:col-span-1"
                    />

                    <RecapStatCard
                        icon={LuZap}
                        color="pink"
                        value={
                            <RecapDurationValue
                                hours={longestSession.hours}
                                minutes={longestSession.minutes}
                            />
                        }
                        label={translation.get('session.longest')}
                        className="col-span-1 lg:col-span-2 xl:col-span-1"
                    />
                </div>
            </div>
        </section>
    );
}
