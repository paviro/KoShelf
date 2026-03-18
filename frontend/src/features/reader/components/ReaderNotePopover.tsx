import { LuNotebookPen, LuX } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

type ReaderNotePopoverProps = {
    open: boolean;
    note: string | null;
    onDismiss: () => void;
};

export function ReaderNotePopover({
    open,
    note,
    onDismiss,
}: ReaderNotePopoverProps) {
    return (
        <ModalShell
            open={open}
            onClose={onDismiss}
            cardClassName="max-w-sm max-h-[70vh] overflow-y-auto bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="relative p-6 pb-4">
                <button
                    type="button"
                    className="absolute top-4 right-4 text-gray-400 dark:text-dark-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-dark-700/50 rounded-full p-2 transition-all duration-200 z-20"
                    title={translation.get('close.aria-label')}
                    aria-label={translation.get('close.aria-label')}
                    onClick={onDismiss}
                >
                    <LuX className="w-5 h-5" aria-hidden="true" />
                </button>

                <div className="flex items-center gap-3 pr-8">
                    <div className="w-10 h-10 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-xl flex items-center justify-center shrink-0">
                        <LuNotebookPen
                            className="w-5 h-5 text-primary-600 dark:text-white"
                            aria-hidden="true"
                        />
                    </div>
                    <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                        {translation.get('my-note')}
                    </h3>
                </div>
            </div>

            <div className="px-6 pb-6">
                <div className="bg-gray-50 dark:bg-dark-800/60 rounded-xl p-4 border border-gray-200/70 dark:border-dark-700/50">
                    <p className="text-gray-700 dark:text-dark-200 leading-relaxed whitespace-pre-wrap">
                        {note}
                    </p>
                </div>
            </div>
        </ModalShell>
    );
}
