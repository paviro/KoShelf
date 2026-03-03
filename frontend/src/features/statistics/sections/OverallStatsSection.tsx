import type { StatisticsIndexResponse } from '../api/statistics-data';
import { LuClock3, LuFileText, LuSun } from 'react-icons/lu';

import { DataFormatter } from '../lib/formatters';
import { translation } from '../../../shared/i18n';
import {
    formatReadTimeWithWeeks,
    formatSessionDuration,
    type SectionName,
} from '../model/statistics-model';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

type OverallStatsSectionProps = {
    visible: boolean;
    onToggle: (sectionName: SectionName) => void;
    overview: StatisticsIndexResponse['overview'];
};

export function OverallStatsSection({ visible, onToggle, overview }: OverallStatsSectionProps) {
    return (
        <CollapsibleSection
            sectionKey="overall-stats"
            accentClass="bg-gradient-to-b from-purple-400 to-purple-600"
            title={translation.get('overall-statistics')}
            visible={visible}
            onToggle={() => onToggle('overall-stats')}
        >
            <div className="grid grid-cols-2 gap-3 sm:gap-4 lg:grid-cols-3 mb-8">
                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600"
                    iconClassName="text-primary-600 dark:text-white"
                    value={formatReadTimeWithWeeks(overview.total_read_time)}
                    label={translation.get('total-read-time')}
                />

                <MetricCard
                    icon={LuFileText}
                    iconContainerClassName="bg-indigo-500/20 dark:bg-gradient-to-br dark:from-indigo-500 dark:to-indigo-600"
                    iconClassName="text-indigo-600 dark:text-white"
                    value={DataFormatter.formatCount(overview.total_page_reads)}
                    label={translation.get('total-pages-read')}
                />

                <MetricCard
                    icon={LuFileText}
                    iconContainerClassName="bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600"
                    iconClassName="text-green-600 dark:text-white"
                    value={DataFormatter.formatCount(overview.most_pages_in_day)}
                    label={translation.get('most-pages-in-day')}
                />

                <MetricCard
                    icon={LuSun}
                    iconContainerClassName="bg-amber-500/20 dark:bg-gradient-to-br dark:from-amber-500 dark:to-amber-600"
                    iconClassName="text-amber-600 dark:text-white"
                    value={DataFormatter.formatReadTime(overview.longest_read_time_in_day)}
                    label={translation.get('longest-daily-reading')}
                />

                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-pink-500/20 dark:bg-gradient-to-br dark:from-pink-500 dark:to-pink-600"
                    iconClassName="text-pink-600 dark:text-white"
                    value={formatSessionDuration(overview.longest_session_duration)}
                    label={translation.get('session.longest')}
                />

                <MetricCard
                    icon={LuClock3}
                    iconContainerClassName="bg-purple-500/20 dark:bg-gradient-to-br dark:from-purple-500 dark:to-purple-600"
                    iconClassName="text-purple-600 dark:text-white"
                    value={formatSessionDuration(overview.average_session_duration)}
                    label={translation.get('session.average')}
                />
            </div>
        </CollapsibleSection>
    );
}
