import { LuInfo, LuSearch } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { PageStateLayout } from '../../../shared/ui/feedback/PageStateLayout';

export function LibraryEmptyState() {
    return (
        <PageStateLayout
            icon={
                <LuSearch className="w-12 h-12 text-white" aria-hidden="true" />
            }
            gradientFrom="from-amber-500"
            gradientTo="to-amber-600"
            glowFrom="from-amber-500/20"
            glowTo="to-amber-600/20"
            title={translation.get('no-books-found')}
            description={translation.get('no-books-match')}
        >
            <div className="flex flex-col sm:flex-row gap-4 items-center mt-8">
                <div className="flex items-center space-x-2 text-sm font-medium text-gray-500 dark:text-dark-400">
                    <LuInfo className="w-4 h-4" aria-hidden="true" />
                    <span>{translation.get('try-adjusting')}</span>
                </div>
            </div>
        </PageStateLayout>
    );
}
