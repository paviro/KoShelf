import {
    LuBookOpen,
    LuClock3,
    LuFlame,
    LuSparkles,
    LuZap,
} from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import type { RecapScope, RecapSummaryResponse } from '../api/recap-data';
import {
    formatRecapMonthLabel,
    formatRecapPercentage,
} from '../lib/recap-formatters';

type RecapSummarySectionProps = {
    year: number;
    scope: RecapScope;
    summary: RecapSummaryResponse;
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

export function RecapSummarySection({
    year,
    scope,
    summary,
}: RecapSummarySectionProps) {
    const daysInYear = isLeapYear(year) ? 366 : 365;
    const bestMonth = summary.best_month_name
        ? formatRecapMonthLabel(summary.best_month_name)
        : null;
    const yearlySummaryLabel = translation.get('yearly-summary', {
        count: String(year),
    });
    const activeDaysPercentage = Number.isFinite(summary.active_days_percentage)
        ? Math.max(0, Math.min(summary.active_days_percentage, 100))
        : 0;
    const activeDaysRingCircumference = 264;
    const activeDaysRingOffset =
        activeDaysRingCircumference -
        (activeDaysRingCircumference * activeDaysPercentage) / 100;

    return (
        <section className="relative pl-10 recap-event">
            <span className="recap-dot recap-dot-top bg-gray-400 dark:bg-dark-400"></span>
            <div className="flex flex-col space-y-3">
                <h3 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                    {yearlySummaryLabel}
                </h3>

                <div className="grid grid-cols-2 lg:grid-cols-6 gap-2 md:gap-3">
                    <div className="col-span-2 lg:col-span-2 lg:row-span-2 bg-gradient-to-br from-teal-500/10 via-teal-400/5 to-transparent dark:from-teal-500/20 dark:via-teal-400/10 dark:to-dark-800 border border-teal-200/50 dark:border-teal-700/30 rounded-xl p-4 shadow-sm relative overflow-hidden group hover:shadow-lg hover:border-teal-300/70 dark:hover:border-teal-600/50 transition-all duration-300">
                        <div className="absolute -top-12 -right-12 w-32 h-32 bg-teal-400/25 dark:bg-teal-400/15 rounded-full blur-2xl group-hover:scale-150 transition-transform duration-500"></div>
                        <div className="absolute -bottom-8 -left-8 w-24 h-24 bg-teal-300/20 dark:bg-teal-500/10 rounded-full blur-xl"></div>

                        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between relative z-10 gap-3 lg:gap-4 h-full">
                            <div className="flex flex-col min-w-0 flex-1 justify-center">
                                <span className="text-xs font-semibold text-teal-600 dark:text-teal-400 uppercase tracking-wider mb-1">
                                    {translation.get(
                                        'active-days',
                                        summary.active_days,
                                    )}
                                </span>
                                <div className="flex items-baseline gap-2 flex-wrap">
                                    <span className="text-4xl sm:text-5xl lg:text-6xl font-black text-gray-900 dark:text-white leading-none">
                                        {formatNumber(summary.active_days)}
                                    </span>
                                    <span className="text-base font-medium text-gray-500 dark:text-gray-400">
                                        {translation.get('of')}{' '}
                                        {formatNumber(daysInYear)}
                                    </span>
                                </div>
                                <div className="flex items-center gap-1.5 mt-2">
                                    <span className="text-xl sm:text-2xl lg:text-3xl font-bold text-teal-600 dark:text-teal-400">
                                        {formatRecapPercentage(
                                            summary.active_days_percentage,
                                        )}
                                        %
                                    </span>
                                    <span className="text-sm text-gray-500 dark:text-gray-400">
                                        {translation.get('of-the-year')}
                                    </span>
                                </div>
                            </div>

                            <div className="hidden xl:block relative w-24 h-24 flex-shrink-0">
                                <svg
                                    className="w-full h-full transform -rotate-90"
                                    viewBox="0 0 100 100"
                                    aria-hidden
                                >
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="42"
                                        fill="none"
                                        stroke="currentColor"
                                        strokeWidth="10"
                                        className="text-gray-200 dark:text-dark-600"
                                    />
                                    <circle
                                        cx="50"
                                        cy="50"
                                        r="42"
                                        fill="none"
                                        stroke="currentColor"
                                        strokeWidth="10"
                                        strokeLinecap="round"
                                        className="text-teal-500 dark:text-teal-400"
                                        strokeDasharray={
                                            activeDaysRingCircumference
                                        }
                                        strokeDashoffset={activeDaysRingOffset}
                                        style={{
                                            transition:
                                                'stroke-dashoffset 0.5s ease-in-out',
                                        }}
                                    />
                                </svg>
                            </div>
                        </div>
                    </div>

                    <div className="col-span-1 lg:col-span-2 bg-gradient-to-br from-blue-500/10 via-blue-400/5 to-transparent dark:from-blue-500/20 dark:via-blue-400/10 dark:to-dark-800 border border-blue-200/50 dark:border-blue-700/30 rounded-xl p-4 shadow-sm relative overflow-hidden group hover:shadow-lg hover:border-blue-300/70 dark:hover:border-blue-600/50 transition-all duration-300">
                        <div className="absolute -bottom-6 -right-6 w-16 h-16 bg-blue-400/20 dark:bg-blue-400/10 rounded-full blur-xl group-hover:scale-150 transition-transform duration-500"></div>
                        <div className="flex items-center gap-3 relative z-10">
                            <div className="w-10 h-10 rounded-xl bg-blue-500/20 dark:bg-gradient-to-br dark:from-blue-500 dark:to-blue-600 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                <LuBookOpen
                                    className="w-5 h-5 text-blue-600 dark:text-white"
                                    aria-hidden
                                />
                            </div>
                            <div className="flex flex-col min-w-0">
                                <span className="text-2xl md:text-3xl font-black text-gray-900 dark:text-white leading-none">
                                    {formatNumber(summary.total_books)}
                                </span>
                                <span className="text-[10px] font-semibold text-blue-600 dark:text-blue-400 uppercase tracking-wider">
                                    {completionLabel(
                                        scope,
                                        summary.total_books,
                                    )}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="col-span-1 lg:col-span-2 bg-gradient-to-br from-purple-500/10 via-purple-400/5 to-transparent dark:from-purple-500/20 dark:via-purple-400/10 dark:to-dark-800 border border-purple-200/50 dark:border-purple-700/30 rounded-xl p-4 shadow-sm relative overflow-hidden group hover:shadow-lg hover:border-purple-300/70 dark:hover:border-purple-600/50 transition-all duration-300">
                        <div className="absolute -top-6 -left-6 w-16 h-16 bg-purple-400/20 dark:bg-purple-400/10 rounded-full blur-xl group-hover:scale-150 transition-transform duration-500"></div>
                        <div className="flex items-center gap-3 relative z-10">
                            <div className="w-10 h-10 rounded-xl bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                <LuClock3
                                    className="w-5 h-5 text-purple-600 dark:text-white"
                                    aria-hidden
                                />
                            </div>
                            <div className="flex flex-col min-w-0">
                                <div className="flex items-baseline gap-1">
                                    {summary.total_time_days > 0 && (
                                        <>
                                            <span className="text-2xl md:text-3xl font-black text-gray-900 dark:text-white leading-none">
                                                {formatNumber(
                                                    summary.total_time_days,
                                                )}
                                            </span>
                                            <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                                                <span className="hidden md:inline">
                                                    {translation.get(
                                                        'days_label',
                                                        summary.total_time_days,
                                                    )}
                                                </span>
                                                <span className="md:hidden">
                                                    {translation.get('units.d')}
                                                </span>
                                            </span>
                                        </>
                                    )}
                                    <span className="text-2xl md:text-3xl font-black text-gray-900 dark:text-white leading-none">
                                        {formatNumber(summary.total_time_hours)}
                                    </span>
                                    <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                                        <span className="hidden md:inline">
                                            {translation.get(
                                                'hours_label',
                                                summary.total_time_hours,
                                            )}
                                        </span>
                                        <span className="md:hidden">
                                            {translation.get('units.h')}
                                        </span>
                                    </span>
                                </div>
                                <span className="text-[10px] font-semibold text-purple-600 dark:text-purple-400 uppercase tracking-wider">
                                    {translation.get('total-read-time')}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="col-span-1 lg:col-span-2 xl:col-span-1 h-full bg-white dark:bg-dark-800/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl px-3 py-2.5 shadow-sm hover:shadow-md hover:border-red-300/50 dark:hover:border-red-700/40 transition-all duration-300 group flex items-center">
                        <div className="flex items-center gap-2.5">
                            <div className="w-8 h-8 rounded-lg bg-red-500/20 dark:bg-gradient-to-br dark:from-red-500 dark:to-orange-500 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                <LuFlame
                                    className="w-4 h-4 text-red-500 dark:text-white"
                                    aria-hidden
                                />
                            </div>
                            <div className="flex flex-col min-w-0">
                                <span className="text-lg font-bold text-gray-900 dark:text-white leading-none">
                                    {formatNumber(summary.longest_streak)}{' '}
                                    <span className="text-xs font-medium text-gray-500 dark:text-gray-400">
                                        <span className="hidden md:inline">
                                            {translation.get(
                                                'days_label',
                                                summary.longest_streak,
                                            )}
                                        </span>
                                        <span className="md:hidden">
                                            {translation.get('units.d')}
                                        </span>
                                    </span>
                                </span>
                                <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                    {translation.get('streak.longest')}
                                </span>
                            </div>
                        </div>
                    </div>

                    {bestMonth && (
                        <div className="col-span-1 lg:col-span-2 xl:col-span-1 h-full bg-white dark:bg-dark-800/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl px-3 py-2.5 shadow-sm hover:shadow-md hover:border-amber-300/50 dark:hover:border-amber-700/40 transition-all duration-300 group flex items-center">
                            <div className="flex items-center gap-2.5">
                                <div className="w-8 h-8 rounded-lg bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-yellow-500 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                    <LuSparkles
                                        className="w-4 h-4 text-amber-500 dark:text-white"
                                        aria-hidden
                                    />
                                </div>
                                <div className="flex flex-col min-w-0">
                                    <span className="text-base font-bold text-gray-900 dark:text-white leading-none truncate">
                                        {bestMonth}
                                    </span>
                                    <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                        {translation.get('best-month')}
                                    </span>
                                </div>
                            </div>
                        </div>
                    )}

                    <div className="col-span-1 lg:col-span-2 xl:col-span-1 h-full bg-white dark:bg-dark-800/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl px-3 py-2.5 shadow-sm hover:shadow-md hover:border-orange-300/50 dark:hover:border-orange-700/40 transition-all duration-300 group flex items-center">
                        <div className="flex items-center gap-2.5">
                            <div className="w-8 h-8 rounded-lg bg-orange-500/20 dark:bg-gradient-to-br dark:from-orange-500 dark:to-amber-500 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                <LuClock3
                                    className="w-4 h-4 text-orange-500 dark:text-white"
                                    aria-hidden
                                />
                            </div>
                            <div className="flex flex-col min-w-0">
                                <div className="flex items-baseline gap-1">
                                    {summary.average_session_hours > 0 && (
                                        <>
                                            <span className="text-lg font-bold text-gray-900 dark:text-white leading-none">
                                                {formatNumber(
                                                    summary.average_session_hours,
                                                )}
                                            </span>
                                            <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400">
                                                <span className="hidden md:inline">
                                                    {translation.get(
                                                        'hours_label',
                                                        summary.average_session_hours,
                                                    )}
                                                </span>
                                                <span className="md:hidden">
                                                    {translation.get('units.h')}
                                                </span>
                                            </span>
                                        </>
                                    )}
                                    {(summary.average_session_hours === 0 ||
                                        summary.average_session_minutes >
                                            0) && (
                                        <>
                                            <span className="text-lg font-bold text-gray-900 dark:text-white leading-none">
                                                {formatNumber(
                                                    summary.average_session_minutes,
                                                )}
                                            </span>
                                            <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400">
                                                <span className="hidden md:inline">
                                                    {translation.get(
                                                        'minutes_label',
                                                        summary.average_session_minutes,
                                                    )}
                                                </span>
                                                <span className="md:hidden">
                                                    {translation.get('units.m')}
                                                </span>
                                            </span>
                                        </>
                                    )}
                                </div>
                                <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                    {translation.get('session.average')}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="col-span-1 lg:col-span-2 xl:col-span-1 h-full bg-white dark:bg-dark-800/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl px-3 py-2.5 shadow-sm hover:shadow-md hover:border-pink-300/50 dark:hover:border-pink-700/40 transition-all duration-300 group flex items-center">
                        <div className="flex items-center gap-2.5">
                            <div className="w-8 h-8 rounded-lg bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-rose-500 flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform duration-300">
                                <LuZap
                                    className="w-4 h-4 text-pink-500 dark:text-white"
                                    aria-hidden
                                />
                            </div>
                            <div className="flex flex-col min-w-0">
                                <div className="flex items-baseline gap-1">
                                    {summary.longest_session_hours > 0 && (
                                        <>
                                            <span className="text-lg font-bold text-gray-900 dark:text-white leading-none">
                                                {formatNumber(
                                                    summary.longest_session_hours,
                                                )}
                                            </span>
                                            <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400">
                                                <span className="hidden md:inline">
                                                    {translation.get(
                                                        'hours_label',
                                                        summary.longest_session_hours,
                                                    )}
                                                </span>
                                                <span className="md:hidden">
                                                    {translation.get('units.h')}
                                                </span>
                                            </span>
                                        </>
                                    )}
                                    {(summary.longest_session_hours === 0 ||
                                        summary.longest_session_minutes >
                                            0) && (
                                        <>
                                            <span className="text-lg font-bold text-gray-900 dark:text-white leading-none">
                                                {formatNumber(
                                                    summary.longest_session_minutes,
                                                )}
                                            </span>
                                            <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400">
                                                <span className="hidden md:inline">
                                                    {translation.get(
                                                        'minutes_label',
                                                        summary.longest_session_minutes,
                                                    )}
                                                </span>
                                                <span className="md:hidden">
                                                    {translation.get('units.m')}
                                                </span>
                                            </span>
                                        </>
                                    )}
                                </div>
                                <span className="text-[10px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                                    {translation.get('session.longest')}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
}
