import { useRef, useState } from 'react';
import { LuCalendarDays, LuChevronDown } from 'react-icons/lu';

import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../dropdown/dropdown-styles';
import { DropdownPortal } from '../dropdown/DropdownPortal';

type YearSelectorProps = {
    idPrefix: string;
    years: number[];
    selectedYear: number | null;
    onSelect: (year: number) => void;
    iconColorClass: string;
    optionActiveClass: string;
    mobileFallback: string;
};

export function YearSelector({
    idPrefix,
    years,
    selectedYear,
    onSelect,
    iconColorClass,
    optionActiveClass,
    mobileFallback,
}: YearSelectorProps) {
    const triggerRef = useRef<HTMLButtonElement>(null);
    const [open, setOpen] = useState(false);

    return (
        <>
            <button
                ref={triggerRef}
                id={`${idPrefix}SelectorWrapper`}
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-controls={`${idPrefix}Options`}
                onClick={() => setOpen((current) => !current)}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} w-10 sm:w-auto sm:px-4`}
            >
                <div className="flex items-center space-x-0 sm:space-x-3">
                    <LuCalendarDays
                        className={`w-5 h-5 ${iconColorClass}`}
                        aria-hidden="true"
                    />
                    <span
                        id={`selected${idPrefix}Text`}
                        className="hidden sm:inline text-gray-900 dark:text-white font-medium text-sm"
                    >
                        {selectedYear ? (
                            <span className="font-bold">{selectedYear}</span>
                        ) : (
                            <span className="font-bold">{mobileFallback}</span>
                        )}
                    </span>
                </div>
                <LuChevronDown
                    className="hidden sm:block w-4 h-4 text-gray-400 dark:text-dark-400 transition-transform duration-200 ml-2"
                    aria-hidden="true"
                />
            </button>

            <DropdownPortal
                triggerRef={triggerRef}
                open={open}
                onClose={() => setOpen(false)}
                closeOnScroll
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} max-h-60 overflow-y-auto w-40`}
            >
                {years.map((year) => {
                    const active = year === selectedYear;
                    return (
                        <button
                            key={year}
                            type="button"
                            className={`w-full text-left px-4 py-2 cursor-pointer hover:bg-gray-100/50 dark:hover:bg-dark-700/50 transition-colors duration-200 ${
                                active
                                    ? optionActiveClass
                                    : 'text-gray-600 dark:text-dark-200 hover:text-gray-900 dark:hover:text-white'
                            }`}
                            onClick={() => {
                                onSelect(year);
                                setOpen(false);
                            }}
                        >
                            <div className="flex items-center">
                                <LuCalendarDays
                                    className="w-4 h-4 text-green-400 mr-2"
                                    aria-hidden="true"
                                />
                                <span className="font-bold">{year}</span>
                            </div>
                        </button>
                    );
                })}
            </DropdownPortal>
        </>
    );
}
