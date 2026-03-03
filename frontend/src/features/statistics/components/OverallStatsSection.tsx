import type { StatisticsIndexResponse } from '../../../shared/statistics-data-loader';
import { DataFormatter } from '../../../shared/statistics-formatters';
import { translation } from '../../../shared/i18n';
import {
    formatReadTimeWithWeeks,
    formatSessionDuration,
    type SectionName,
} from '../model/statistics-model';
import { StatBadgeCard } from './StatBadgeCard';
import { StatisticsSection } from './StatisticsSection';

const CLOCK_ICON = 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z';
const FILE_ICON =
    'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z';
const SUN_ICON =
    'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z';

type OverallStatsSectionProps = {
    visible: boolean;
    onToggle: (sectionName: SectionName) => void;
    overview: StatisticsIndexResponse['overview'];
};

export function OverallStatsSection({ visible, onToggle, overview }: OverallStatsSectionProps) {
    return (
        <StatisticsSection
            sectionName="overall-stats"
            accentClass="bg-gradient-to-b from-purple-400 to-purple-600"
            title={translation.get('overall-statistics')}
            visible={visible}
            onToggle={() => onToggle('overall-stats')}
        >
            <div className="grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3 mb-8">
                <StatBadgeCard
                    iconPath={CLOCK_ICON}
                    iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                    iconClassName="text-primary-600 dark:text-white"
                    value={formatReadTimeWithWeeks(overview.total_read_time)}
                    label={translation.get('total-read-time')}
                />

                <StatBadgeCard
                    iconPath={FILE_ICON}
                    iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                    iconClassName="text-indigo-600 dark:text-white"
                    value={overview.total_page_reads}
                    label={translation.get('total-pages-read')}
                />

                <StatBadgeCard
                    iconPath={FILE_ICON}
                    iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                    iconClassName="text-green-600 dark:text-white"
                    value={overview.most_pages_in_day}
                    label={translation.get('most-pages-in-day')}
                />

                <StatBadgeCard
                    iconPath={SUN_ICON}
                    iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                    iconClassName="text-amber-600 dark:text-white"
                    value={DataFormatter.formatReadTime(overview.longest_read_time_in_day)}
                    label={translation.get('longest-daily-reading')}
                />

                <StatBadgeCard
                    iconPath={CLOCK_ICON}
                    iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                    iconClassName="text-pink-600 dark:text-white"
                    value={formatSessionDuration(overview.longest_session_duration)}
                    label={translation.get('session.longest')}
                />

                <StatBadgeCard
                    iconPath={CLOCK_ICON}
                    iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                    iconClassName="text-purple-600 dark:text-white"
                    value={formatSessionDuration(overview.average_session_duration)}
                    label={translation.get('session.average')}
                />
            </div>
        </StatisticsSection>
    );
}
