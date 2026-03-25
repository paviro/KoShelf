import { LuChartColumnIncreasing } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { InfoDetails } from '../../../shared/ui/feedback/InfoDetails';
import { PageStateLayout } from '../../../shared/ui/feedback/PageStateLayout';

export function StatisticsEmptyState() {
    return (
        <PageStateLayout
            icon={
                <LuChartColumnIncreasing
                    className="w-12 h-12 text-white"
                    aria-hidden
                />
            }
            gradientFrom="from-green-500"
            gradientTo="to-emerald-500"
            glowFrom="from-green-500/20"
            glowTo="to-emerald-500/20"
            title={translation.get('stats-empty.nothing-here')}
            description={translation.get('stats-empty.start-reading')}
            id="dynamicEmptyState"
        >
            <InfoDetails
                question={translation.get('stats-empty.info-question')}
                answer={translation.get('stats-empty.info-answer')}
            />
        </PageStateLayout>
    );
}
