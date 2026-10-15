import { LuArchive } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';

export function ArchivedBadge({
    variant = 'cover',
}: {
    variant?: 'cover' | 'heading';
}) {
    return (
        <span
            className={`inline-flex items-center gap-1.5 py-1 font-medium whitespace-nowrap rounded-lg border ${variant === 'heading' ? 'text-xs px-2.5 bg-gray-100/50 dark:bg-dark-800/50 text-gray-600 dark:text-dark-300 border-gray-300/50 dark:border-dark-700/50' : 'text-xs px-2.5 bg-linear-to-br from-gray-600 to-gray-700 text-white shadow-lg backdrop-blur-xs border-gray-400/30'}`}
        >
            <LuArchive className="w-3 h-3 shrink-0" aria-hidden="true" />
            {translation.get('library-archived')}
        </span>
    );
}
