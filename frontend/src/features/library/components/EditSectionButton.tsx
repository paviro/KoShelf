import { LuPencil } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';

type EditSectionButtonProps = {
    editing: boolean;
    onToggle: () => void;
};

export function EditSectionButton({
    editing,
    onToggle,
}: EditSectionButtonProps) {
    return (
        <button
            type="button"
            onClick={onToggle}
            className={`flex items-center justify-center w-10 h-10 rounded-lg border transition-colors backdrop-blur-xs ${
                editing
                    ? 'bg-primary-50 dark:bg-primary-500/10 border-primary-300/50 dark:border-primary-500/30 text-primary-600 dark:text-primary-400'
                    : 'bg-gray-100/50 dark:bg-dark-800/50 border-gray-300/50 dark:border-dark-700/50 text-gray-600 dark:text-dark-300 hover:bg-gray-200/50 dark:hover:bg-dark-700/50'
            }`}
            aria-label={translation.get('edit.aria-label')}
            aria-pressed={editing}
        >
            <LuPencil className="w-4 h-4" aria-hidden="true" />
        </button>
    );
}
