import { useRef, useState } from 'react';

import { useClickOutside } from '../../../shared/hooks/useClickOutside';

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
    const wrapperRef = useRef<HTMLDivElement>(null);
    const [open, setOpen] = useState(false);
    useClickOutside(wrapperRef, () => setOpen(false), open);

    return (
        <div className="relative" ref={wrapperRef}>
            <button
                id={`${idPrefix}SelectorWrapper`}
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-controls={`${idPrefix}Options`}
                onClick={() => setOpen((current) => !current)}
                className="dropdown-trigger flex items-center justify-center sm:justify-between bg-gray-100/50 dark:bg-dark-800/50 border border-gray-300/50 dark:border-dark-700/50 rounded-lg w-10 sm:w-auto sm:px-4 h-10 cursor-pointer hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-all duration-200 text-sm md:text-base backdrop-blur-sm"
            >
                <div className="flex items-center space-x-0 sm:space-x-3">
                    <svg
                        className={`w-5 h-5 ${iconColorClass}`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                        ></path>
                    </svg>
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
                <svg
                    className="hidden sm:block w-4 h-4 text-gray-400 dark:text-dark-400 transition-transform duration-200 ml-2"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth="2"
                        d="M19 9l-7 7-7-7"
                    ></path>
                </svg>
            </button>

            <div
                id={`${idPrefix}Options`}
                className={`dropdown-menu-right z-30 max-h-60 overflow-y-auto bg-white dark:bg-dark-800/75 border border-gray-200/50 dark:border-dark-700/50 rounded-lg shadow-xl w-40 overflow-hidden backdrop-blur-md ${open ? '' : 'hidden'}`}
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
                            <div className="flex items-center justify-between">
                                <div className="flex items-center">
                                    <svg
                                        className="w-4 h-4 text-green-400 mr-2"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path
                                            strokeLinecap="round"
                                            strokeLinejoin="round"
                                            strokeWidth="2"
                                            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                                        ></path>
                                    </svg>
                                    <span className="font-bold">{year}</span>
                                </div>
                            </div>
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
