import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { formatRecapPercentage } from '../lib/recap-formatters';

type RecapActiveDaysCardProps = {
    activeDays: number;
    daysInYear: number;
    activeDaysPercentage: number;
    className?: string;
};

const RING_CIRCUMFERENCE = 264;

export function RecapActiveDaysCard({
    activeDays,
    daysInYear,
    activeDaysPercentage,
    className,
}: RecapActiveDaysCardProps) {
    const safePercentage = Number.isFinite(activeDaysPercentage)
        ? Math.max(0, Math.min(activeDaysPercentage, 100))
        : 0;
    const ringOffset =
        RING_CIRCUMFERENCE - (RING_CIRCUMFERENCE * safePercentage) / 100;

    return (
        <div
            className={`@container bg-linear-to-br from-teal-500/10 via-teal-400/5 to-transparent dark:from-teal-500/20 dark:via-teal-400/10 dark:to-dark-800 border border-teal-200/50 dark:border-teal-700/30 rounded-xl p-4 shadow-xs relative overflow-hidden group hover:shadow-lg hover:border-teal-300/70 dark:hover:border-teal-600/50 transition-all duration-300${className ? ` ${className}` : ''}`}
        >
            <div className="absolute -top-12 -right-12 w-32 h-32 bg-teal-400/25 dark:bg-teal-400/15 rounded-full blur-2xl group-hover:scale-150 transition-transform duration-500"></div>
            <div className="absolute -bottom-8 -left-8 w-24 h-24 bg-teal-300/20 dark:bg-teal-500/10 rounded-full blur-xl"></div>

            <div className="flex flex-col @[260px]:flex-row @[260px]:items-center @[260px]:justify-between relative z-10 gap-3 @[260px]:gap-4 h-full">
                <div className="flex flex-col min-w-0 flex-1 justify-center">
                    <span className="text-xs font-semibold text-teal-600 dark:text-teal-400 uppercase tracking-wider mb-1">
                        {translation.get('active-days', activeDays)}
                    </span>
                    <div className="flex items-baseline gap-2 flex-wrap">
                        <span className="text-4xl sm:text-5xl lg:text-6xl font-black text-gray-900 dark:text-white leading-none">
                            {formatNumber(activeDays)}
                        </span>
                        <span className="text-base font-medium text-gray-500 dark:text-gray-400">
                            {translation.get('of')} {formatNumber(daysInYear)}
                        </span>
                    </div>
                    <div className="flex items-center gap-1.5 mt-2">
                        <span className="text-xl sm:text-2xl lg:text-3xl font-bold text-teal-600 dark:text-teal-400">
                            {formatRecapPercentage(activeDaysPercentage)}%
                        </span>
                        <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
                            {translation.get('of-the-year')}
                        </span>
                    </div>
                </div>

                <div className="hidden @[260px]:block relative w-24 h-24 shrink-0">
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
                            strokeDasharray={RING_CIRCUMFERENCE}
                            strokeDashoffset={ringOffset}
                            style={{
                                transition:
                                    'stroke-dashoffset 0.5s ease-in-out',
                            }}
                        />
                    </svg>
                </div>
            </div>
        </div>
    );
}
