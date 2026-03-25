import { LuScrollText } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { InfoDetails } from '../../../shared/ui/feedback/InfoDetails';
import { PageStateLayout } from '../../../shared/ui/feedback/PageStateLayout';

type RecapEmptyStateProps = {
    hasYearContext: boolean;
};

export function RecapEmptyState({ hasYearContext }: RecapEmptyStateProps) {
    return (
        <PageStateLayout
            icon={<LuScrollText className="w-12 h-12 text-white" aria-hidden />}
            gradientFrom="from-purple-500"
            gradientTo="to-pink-500"
            glowFrom="from-purple-500/20"
            glowTo="to-pink-500/20"
            title={translation.get('recap-empty.nothing-here')}
            description={
                hasYearContext
                    ? translation.get('recap-empty.try-switching')
                    : translation.get('recap-empty.finish-reading')
            }
        >
            <InfoDetails
                question={translation.get('recap-empty.info-question')}
                answer={translation.get('recap-empty.info-answer')}
            />
        </PageStateLayout>
    );
}
