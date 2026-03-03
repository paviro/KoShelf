import { useMemo } from 'react';

import { translation } from '../../../shared/i18n';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

type CalendarMonthPickerModalProps = {
    open: boolean;
    year: number;
    selectedMonthIndex: number;
    locale: string;
    onClose: () => void;
    onSelectMonth: (monthIndex: number) => void;
};

export function CalendarMonthPickerModal({
    open,
    year,
    selectedMonthIndex,
    locale,
    onClose,
    onSelectMonth,
}: CalendarMonthPickerModalProps) {
    const monthFormatter = useMemo(
        () =>
            new Intl.DateTimeFormat(locale || 'en', {
                month: 'short',
            }),
        [locale],
    );

    return (
        <ModalShell
            open={open}
            onClose={onClose}
            cardClassName="max-w-xs bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl overflow-hidden"
            showCloseButton={false}
        >
            <div className="p-4 bg-gradient-to-b from-white/70 to-transparent dark:from-white/[0.02] dark:to-transparent">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 text-center">
                    {translation.get('select-month')}
                </h3>
                <div className="grid grid-cols-3 gap-2">
                    {Array.from({ length: 12 }, (_, monthIndex) => {
                        const monthName = monthFormatter.format(new Date(year, monthIndex, 1));
                        const active = monthIndex === selectedMonthIndex;

                        return (
                            <button
                                key={`${year}-${monthIndex}`}
                                type="button"
                                className={`px-3 py-2 text-sm rounded-lg transition-colors duration-200 ${
                                    active
                                        ? 'bg-gradient-to-r from-primary-600 to-primary-700 text-white shadow-md'
                                        : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100/80 dark:hover:bg-dark-700/70'
                                }`}
                                onClick={() => {
                                    onSelectMonth(monthIndex);
                                    onClose();
                                }}
                            >
                                {monthName}
                            </button>
                        );
                    })}
                </div>
            </div>
        </ModalShell>
    );
}
