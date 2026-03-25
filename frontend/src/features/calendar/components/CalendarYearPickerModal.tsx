import { LuChevronLeft, LuChevronRight } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
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
            <div className="p-4 bg-linear-to-b from-white/70 to-transparent dark:from-white/2 dark:to-transparent">
                <div className="flex items-center justify-between mb-4">
                    <Button
                        variant="neutral"
                        size="icon"
                        icon={LuChevronLeft}
                        label={translation.get('previous-month.aria-label')}
                        onClick={onPreviousRange}
                    />
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                        {rangeStartYear} - {rangeStartYear + 8}
                    </h3>
                    <Button
                        variant="neutral"
                        size="icon"
                        icon={LuChevronRight}
                        label={translation.get('next-month.aria-label')}
                        onClick={onNextRange}
                    />
                </div>
                <div className="grid grid-cols-3 gap-2">
                    {Array.from(
                        { length: 9 },
                        (_, index) => rangeStartYear + index,
                    ).map((year) => {
                        const active = year === selectedYear;
                        return (
                            <Button
                                key={year}
                                variant={active ? 'gradient' : 'ghost'}
                                size="xs"
                                className={
                                    active
                                        ? 'px-3 py-2'
                                        : 'px-3 py-2 text-gray-700 dark:text-gray-200'
                                }
                                onClick={() => {
                                    onSelectYear(year);
                                    onClose();
                                }}
                            >
                                {year}
                            </Button>
                        );
                    })}
                </div>
            </div>
        </ModalShell>
    );
}
