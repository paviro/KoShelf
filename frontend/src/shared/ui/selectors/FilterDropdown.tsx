import { useRef, useState } from 'react';
import { LuChevronDown, LuFilter } from 'react-icons/lu';

import { useClickOutside } from '../../lib/dom/useClickOutside';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../dropdown/dropdown-styles';
import { DropdownOption } from '../dropdown/DropdownOption';

export type FilterDropdownOption<T extends string> = {
    value: T;
    label: string;
};

type FilterDropdownProps<T extends string> = {
    value: T;
    options: readonly FilterDropdownOption<T>[];
    onChange: (value: T) => void;
    ariaLabel: string;
    panelClassName?: string;
    separateOptions?: boolean;
    optionClassName?: string;
};

export function FilterDropdown<T extends string>({
    value,
    options,
    onChange,
    ariaLabel,
    panelClassName = 'w-40',
    separateOptions = false,
    optionClassName,
}: FilterDropdownProps<T>) {
    const wrapperRef = useRef<HTMLDivElement>(null);
    const [open, setOpen] = useState(false);

    useClickOutside(wrapperRef, () => setOpen(false), open);

    const activeLabel = options.find((o) => o.value === value)?.label ?? value;

    return (
        <div className="relative" ref={wrapperRef}>
            <button
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-label={ariaLabel}
                title={ariaLabel}
                onClick={() => setOpen((current) => !current)}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} text-gray-900 dark:text-white sm:gap-2 sm:justify-start w-10 sm:w-auto sm:px-4 sm:pl-6`}
            >
                <LuFilter
                    className={`sm:hidden w-5 h-5 ${value === 'all' ? 'text-gray-600 dark:text-gray-300' : 'text-primary-500'}`}
                    aria-hidden="true"
                />
                <span className="hidden sm:inline font-medium">
                    {activeLabel}
                </span>
                <LuChevronDown
                    className="hidden sm:block w-4 h-4 text-primary-400"
                    aria-hidden="true"
                />
            </button>

            <div
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} right-0 ${panelClassName} ${open ? '' : 'hidden'}`}
                role="menu"
            >
                {options.map((option, index) => (
                    <DropdownOption
                        key={option.value}
                        active={option.value === value}
                        separator={
                            separateOptions && index < options.length - 1
                        }
                        className={optionClassName}
                        onClick={() => {
                            onChange(option.value);
                            setOpen(false);
                        }}
                    >
                        {option.label}
                    </DropdownOption>
                ))}
            </div>
        </div>
    );
}
