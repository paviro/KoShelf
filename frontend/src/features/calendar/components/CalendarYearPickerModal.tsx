import { LuChevronLeft, LuChevronRight } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

type CalendarYearPickerModalProps = {
    open: boolean;
    selectedYear: number;
    rangeStartYear: number;
    onClose: () => void;
    onPreviousRange: () => void;
    onNextRange: () => void;
    onSelectYear: (year: number) => void;
};

export function CalendarYearPickerModal({
    open,
    selectedYear,
    rangeStartYear,
    onClose,
    onPreviousRange,
    onNextRange,
    onSelectYear,
}: CalendarYearPickerModalProps) {
    return (
        <ModalShell
            open={open}
            onClose={onClose}
            cardClassName="max-w-xs bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl overflow-hidden"
            showCloseButton={false}
        >
            <div className="p-4 bg-gradient-to-b from-white/70 to-transparent dark:from-white/[0.02] dark:to-transparent">
                <div className="flex items-center justify-between mb-4">
                    <button
                        type="button"
                        className="p-2 rounded-lg border border-gray-200/70 dark:border-dark-700/60 bg-gray-100/60 dark:bg-dark-800/40 hover:bg-gray-200/60 dark:hover:bg-dark-700/60 transition-colors"
                        onClick={onPreviousRange}
                        aria-label={translation.get('previous-month.aria-label')}
                    >
                        <LuChevronLeft className="w-5 h-5 text-gray-600 dark:text-gray-300" />
                    </button>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {rangeStartYear} - {rangeStartYear + 8}
                    </h3>
                    <button
                        type="button"
                        className="p-2 rounded-lg border border-gray-200/70 dark:border-dark-700/60 bg-gray-100/60 dark:bg-dark-800/40 hover:bg-gray-200/60 dark:hover:bg-dark-700/60 transition-colors"
                        onClick={onNextRange}
                        aria-label={translation.get('next-month.aria-label')}
                    >
                        <LuChevronRight className="w-5 h-5 text-gray-600 dark:text-gray-300" />
                    </button>
                </div>
                <div className="grid grid-cols-3 gap-2">
                    {Array.from({ length: 9 }, (_, index) => rangeStartYear + index).map((year) => {
                        const active = year === selectedYear;
                        return (
                            <button
                                key={year}
                                type="button"
                                className={`px-3 py-2 text-sm rounded-lg transition-colors duration-200 ${
                                    active
                                        ? 'bg-gradient-to-r from-primary-600 to-primary-700 text-white shadow-md'
                                        : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100/80 dark:hover:bg-dark-700/70'
                                }`}
                                onClick={() => {
                                    onSelectYear(year);
                                    onClose();
                                }}
                            >
                                {year}
                            </button>
                        );
                    })}
                </div>
            </div>
        </ModalShell>
    );
}
