import { useState } from 'react';
import { LuTriangleAlert } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

type EditWarningModalProps = {
    open: boolean;
    onAcknowledge: (dontShowAgain: boolean) => void;
    onCancel: () => void;
};

export function EditWarningModal({
    open,
    onAcknowledge,
    onCancel,
}: EditWarningModalProps) {
    const [dontShow, setDontShow] = useState(false);

    // Reset checkbox when modal reopens so a previous Cancel doesn't persist it.
    const [prevOpen, setPrevOpen] = useState(false);
    if (open && !prevOpen) {
        setDontShow(false);
    }
    if (open !== prevOpen) {
        setPrevOpen(open);
    }

    return (
        <ModalShell
            open={open}
            onClose={onCancel}
            containerClassName="z-[60]"
            cardClassName="max-w-md bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="p-6">
                <div className="flex items-center gap-3 mb-4">
                    <div className="w-10 h-10 rounded-full bg-amber-100 dark:bg-amber-500/20 flex items-center justify-center shrink-0">
                        <LuTriangleAlert className="w-5 h-5 text-amber-600 dark:text-amber-400" />
                    </div>
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {translation.get('edit-warning.title')}
                    </h2>
                </div>

                <p className="text-sm text-gray-600 dark:text-dark-300 leading-relaxed mb-6">
                    {translation.get('edit-warning.body')}
                </p>

                <label className="flex items-center gap-2 mb-6 cursor-pointer select-none">
                    <input
                        type="checkbox"
                        checked={dontShow}
                        onChange={(e) => setDontShow(e.target.checked)}
                        className="w-4 h-4 rounded border-gray-300 dark:border-dark-600 text-primary-600 focus:ring-primary-500"
                    />
                    <span className="text-sm text-gray-500 dark:text-dark-400">
                        {translation.get('edit-warning.dismiss')}
                    </span>
                </label>

                <div className="flex justify-end gap-2">
                    <button
                        type="button"
                        onClick={onCancel}
                        className="px-4 py-2 text-sm text-gray-500 dark:text-dark-400 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                    >
                        {translation.get('cancel')}
                    </button>
                    <button
                        type="button"
                        onClick={() => onAcknowledge(dontShow)}
                        className="px-4 py-2 text-sm font-medium text-primary-600 dark:text-primary-400 border border-primary-500/30 dark:border-primary-500/20 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-500/10 transition-colors"
                    >
                        {translation.get('edit-warning.understood')}
                    </button>
                </div>
            </div>
        </ModalShell>
    );
}
