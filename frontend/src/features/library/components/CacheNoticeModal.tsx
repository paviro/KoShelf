import { useState } from 'react';
import { LuInfo } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

type CacheNoticeModalProps = {
    open: boolean;
    onClose: (dontShowAgain: boolean) => void;
};

const STEPS = [
    'koreader-cache-notice.step-refresh',
    'koreader-cache-notice.step-bookshelf',
    'koreader-cache-notice.step-restart',
    'koreader-cache-notice.step-repeat',
];

export function CacheNoticeModal({ open, onClose }: CacheNoticeModalProps) {
    const [dontShow, setDontShow] = useState(false);

    const handleClose = () => {
        const dontShowAgain = dontShow;
        setDontShow(false);
        onClose(dontShowAgain);
    };

    return (
        <ModalShell
            open={open}
            onClose={handleClose}
            containerClassName="z-[60]"
            cardClassName="max-w-md bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="p-6">
                <div className="flex items-center gap-3 mb-4">
                    <div className="w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-500/20 flex items-center justify-center shrink-0">
                        <LuInfo className="w-5 h-5 text-primary-600 dark:text-primary-400" />
                    </div>
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {translation.get('koreader-cache-notice.title')}
                    </h2>
                </div>

                <p className="text-sm font-medium text-gray-600 dark:text-dark-300 leading-relaxed mb-3">
                    {translation.get('koreader-cache-notice.body')}
                </p>
                <ol className="list-decimal pl-5 space-y-1.5 mb-6 text-sm text-gray-600 dark:text-dark-300 leading-relaxed">
                    {STEPS.map((key) => (
                        <li key={key}>{translation.get(key)}</li>
                    ))}
                </ol>

                <label className="flex items-center gap-2 mb-6 cursor-pointer select-none">
                    <input
                        type="checkbox"
                        checked={dontShow}
                        onChange={(e) => setDontShow(e.target.checked)}
                        className="w-4 h-4 rounded border-gray-300 dark:border-dark-600 text-primary-600 focus:ring-primary-500"
                    />
                    <span className="text-sm font-medium text-gray-500 dark:text-dark-400">
                        {translation.get('koreader-cache-notice.dismiss')}
                    </span>
                </label>

                <div className="flex justify-end">
                    <Button onClick={handleClose}>
                        {translation.get('koreader-cache-notice.ok')}
                    </Button>
                </div>
            </div>
        </ModalShell>
    );
}
