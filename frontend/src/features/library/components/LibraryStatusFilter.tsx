import { useRef, useState } from 'react';
import { LuChevronDown, LuFilter } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { useClickOutside } from '../../../shared/lib/dom/useClickOutside';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../../../shared/ui/dropdown/dropdown-styles';
import { DropdownOption } from '../../../shared/ui/dropdown/DropdownOption';
import type { LibraryFilterValue } from '../model/library-model';

type LibraryStatusFilterProps = {
    value: LibraryFilterValue;
    options: readonly LibraryFilterValue[];
    onChange: (filter: LibraryFilterValue) => void;
};

const FILTER_LABEL_KEYS: Record<LibraryFilterValue, string> = {
    all: 'filter.all',
    reading: 'filter.reading',
    completed: 'filter.completed',
    abandoned: 'status.on-hold',
    unread: 'filter.unread',
};

const FILTER_ARIA_KEYS: Record<LibraryFilterValue, string> = {
    all: 'filter.all-aria',
    reading: 'filter.reading-aria',
    completed: 'filter.completed-aria',
    abandoned: 'filter.on-hold-aria',
    unread: 'filter.unread-aria',
};

function filterLabel(filter: LibraryFilterValue): string {
    return translation.get(FILTER_LABEL_KEYS[filter]);
}

function filterAriaLabel(filter: LibraryFilterValue): string {
    return translation.get(FILTER_ARIA_KEYS[filter]);
}

export function LibraryStatusFilter({
    value,
    options,
    onChange,
}: LibraryStatusFilterProps) {
    const wrapperRef = useRef<HTMLDivElement>(null);
    const [open, setOpen] = useState(false);

    useClickOutside(wrapperRef, () => setOpen(false), open);

    return (
        <div className="relative" ref={wrapperRef}>
            <button
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-label={filterAriaLabel(value)}
                title={filterAriaLabel(value)}
                onClick={() => setOpen((current) => !current)}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} text-gray-900 dark:text-white space-x-0 sm:space-x-2 sm:justify-start w-10 sm:w-auto sm:px-4`}
            >
                <LuFilter
                    className={`sm:hidden w-5 h-5 ${value === 'all' ? 'text-gray-600 dark:text-gray-300' : 'text-primary-500'}`}
                    aria-hidden="true"
                />
                <span className="hidden sm:inline font-medium">
                    {filterLabel(value)}
                </span>
                <LuChevronDown
                    className="hidden sm:block w-4 h-4 text-primary-400"
                    aria-hidden="true"
                />
            </button>

            <div
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} right-0 w-max min-w-40 ${open ? '' : 'hidden'}`}
                role="menu"
            >
                {options.map((option, index) => (
                    <DropdownOption
                        key={option}
                        active={option === value}
                        separator={index < options.length - 1}
                        className="text-sm md:text-base whitespace-nowrap"
                        onClick={() => {
                            onChange(option);
                            setOpen(false);
                        }}
                    >
                        {filterLabel(option)}
                    </DropdownOption>
                ))}
            </div>
        </div>
    );
}
