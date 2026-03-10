import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuCalendarDays, LuClock3, LuInfo, LuZap } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { MetricCardUnitValue } from '../../../shared/ui/cards/MetricCardUnitValue';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import type {
    LibraryCompletions,
    LibraryItemStats,
    LibrarySessionStats,
} from '../api/library-data';
import {
    calculateAverageReadingSpeed,
    calculateCalendarLengthDays,
    formatCompletionDateRange,
    formatDurationFromSeconds,
    formatDurationFromSecondsParts,
    formatIsoDate,
    formatReadingSpeed,
} from '../lib/library-detail-formatters';

type LibraryReadingStatsSectionProps = {
    itemStats: LibraryItemStats | null;
    sessionStats: LibrarySessionStats;
    completions: LibraryCompletions | null;
    visible: boolean;
    onToggle: () => void;
};

export function LibraryReadingStatsSection({
    itemStats,
    sessionStats,
    completions,
    visible,
    onToggle,
}: LibraryReadingStatsSectionProps) {
    return (
        <CollapsibleSection
            sectionKey="reading-stats"
            defaultVisible={false}
            accentClass="bg-gradient-to-b from-blue-400 to-blue-600"
            title={translation.get('reading-statistics')}
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
        >
            <div className="grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3">
                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                    iconClassName="text-primary-600 dark:text-white"
                    value={
                        <MetricCardUnitValue
                            value={formatDurationFromSecondsParts(
                                itemStats?.total_read_time,
                            )}
                        />
                    }
                    label={translation.get('total-read-time')}
                />

                <MetricCard
                    icon={HiOutlineBookOpen}
                    iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                    iconClassName="text-indigo-600 dark:text-white"
                    value={formatNumber(sessionStats.session_count)}
                    label={translation.get(
                        'reading-sessions-label',
                        sessionStats.session_count,
                    )}
                />

                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                    iconClassName="text-green-600 dark:text-white"
                    value={
                        <MetricCardUnitValue
                            value={formatDurationFromSecondsParts(
                                sessionStats.average_session_duration,
                            )}
                        />
                    }
                    label={translation.get('session.average')}
                />

                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                    iconClassName="text-pink-600 dark:text-white"
                    value={
                        <MetricCardUnitValue
                            value={formatDurationFromSecondsParts(
                                sessionStats.longest_session_duration,
                            )}
                        />
                    }
                    label={translation.get('session.longest')}
                />

                <MetricCard
                    icon={LuZap}
                    iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                    iconClassName="text-amber-600 dark:text-white"
                    value={formatReadingSpeed(sessionStats.reading_speed)}
                    label={translation.get('pages-per-hour')}
                />

                <MetricCard
                    icon={LuCalendarDays}
                    iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                    iconClassName="text-purple-600 dark:text-white"
                    value={formatIsoDate(sessionStats.last_read_date)}
                    label={translation.get('last-read')}
                />
            </div>

            {completions && completions.total_completions > 0 && (
                <div className="mt-8">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                        <div className="w-2 h-5 bg-gradient-to-b from-emerald-400 to-emerald-600 rounded-full mr-3"></div>
                        {translation.get('reading-completions')}
                    </h3>

                    <div className="space-y-3">
                        {completions.entries.map((entry, index) => {
                            const averageSessionDuration =
                                entry.session_count > 0
                                    ? Math.floor(
                                          entry.reading_time /
                                              entry.session_count,
                                      )
                                    : null;
                            const averageSpeed = calculateAverageReadingSpeed(
                                entry.pages_read,
                                entry.reading_time,
                            );
                            const calendarLength = calculateCalendarLengthDays(
                                entry.start_date,
                                entry.end_date,
                            );

                            return (
                                <article
                                    key={`${entry.start_date}-${entry.end_date}-${index}`}
                                    className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4"
                                >
                                    <div className="flex items-center space-x-4">
                                        <div className="w-10 h-10 bg-gradient-to-br from-primary-500/10 to-primary-600/10 rounded-lg flex items-center justify-center flex-shrink-0">
                                            <LuCalendarDays
                                                className="w-5 h-5 text-primary-600 dark:text-primary-400"
                                                aria-hidden="true"
                                            />
                                        </div>

                                        <div className="flex-1 min-w-0">
                                            <div className="text-sm font-medium text-gray-900 dark:text-white mb-1">
                                                {formatCompletionDateRange(
                                                    entry.start_date,
                                                    entry.end_date,
                                                )}
                                            </div>

                                            <div className="flex flex-wrap items-center mt-1 gap-2 text-xs text-gray-500 dark:text-dark-400">
                                                <span className="flex items-center whitespace-nowrap">
                                                    <LuClock3
                                                        className="w-3.5 h-3.5 mr-1"
                                                        aria-hidden="true"
                                                    />
                                                    {formatDurationFromSeconds(
                                                        entry.reading_time,
                                                    )}
                                                </span>

                                                <span className="flex items-center whitespace-nowrap">
                                                    <HiOutlineBookOpen
                                                        className="w-3.5 h-3.5 mr-1"
                                                        aria-hidden="true"
                                                    />
                                                    {translation.get(
                                                        'units-sessions',
                                                        entry.session_count,
                                                    )}
                                                </span>

                                                {averageSessionDuration !==
                                                    null && (
                                                    <span className="flex items-center whitespace-nowrap">
                                                        <LuClock3
                                                            className="w-3.5 h-3.5 mr-1"
                                                            aria-hidden="true"
                                                        />
                                                        {formatDurationFromSeconds(
                                                            averageSessionDuration,
                                                        )}
                                                        {translation.get(
                                                            'avg-session-suffix',
                                                        )}
                                                    </span>
                                                )}

                                                {averageSpeed !== null && (
                                                    <span className="flex items-center whitespace-nowrap">
                                                        <LuZap
                                                            className="w-3.5 h-3.5 mr-1"
                                                            aria-hidden="true"
                                                        />
                                                        {formatReadingSpeed(
                                                            averageSpeed,
                                                        )}{' '}
                                                        {translation.get(
                                                            'pph-abbreviation',
                                                        )}
                                                    </span>
                                                )}

                                                {calendarLength !== null && (
                                                    <span className="flex items-center whitespace-nowrap">
                                                        <LuCalendarDays
                                                            className="w-3.5 h-3.5 mr-1"
                                                            aria-hidden="true"
                                                        />
                                                        {translation.get(
                                                            'units-days',
                                                            calendarLength,
                                                        )}
                                                    </span>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </article>
                            );
                        })}
                    </div>
                </div>
            )}

            <div className="mt-6 p-4 bg-primary-50 dark:bg-dark-850/30 rounded-lg border border-primary-200 dark:border-dark-700/50">
                <div className="flex items-center text-sm text-gray-500 dark:text-dark-400">
                    <LuInfo
                        className="w-4 h-4 mr-2 text-primary-400"
                        aria-hidden="true"
                    />
                    {translation.get('statistics-from-koreader')}
                </div>
            </div>
        </CollapsibleSection>
    );
}
