import { useRef, useState } from 'react';
import { LuCheck, LuChevronDown, LuFilter } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { DropdownOption } from '../../../shared/ui/dropdown/DropdownOption';
import { DropdownPortal } from '../../../shared/ui/dropdown/DropdownPortal';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../../../shared/ui/dropdown/dropdown-styles';
import type {
    LibraryAvailability,
    LibraryFilterValue,
} from '../model/library-model';

type LibraryStatusFilterProps = {
    value: LibraryFilterValue;
    options: readonly LibraryFilterValue[];
    onChange: (filter: LibraryFilterValue) => void;
    archiveEnabled: boolean;
    availability: LibraryAvailability;
    onAvailabilityChange: (availability: LibraryAvailability) => void;
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

const AVAILABILITY_LABEL_KEYS: Record<LibraryAvailability, string> = {
    all: 'library-all',
    available: 'library-available',
    archived: 'library-archived',
};
const AVAILABILITY_OPTIONS: readonly LibraryAvailability[] = [
    'all',
    'available',
    'archived',
];

export function LibraryStatusFilter({
    value,
    options,
    onChange,
    archiveEnabled,
    availability,
    onAvailabilityChange,
}: LibraryStatusFilterProps) {
    const triggerRef = useRef<HTMLButtonElement>(null);
    const [open, setOpen] = useState(false);
    const statusLabel = translation.get(FILTER_LABEL_KEYS[value]);
    const availabilityLabel = translation.get(
        AVAILABILITY_LABEL_KEYS[availability],
    );
    const active = value !== 'all' || availability !== 'all';
    const label =
        availability === 'all'
            ? statusLabel
            : value === 'all'
              ? availabilityLabel
              : `${statusLabel} · ${availabilityLabel}`;
    const ariaLabel = archiveEnabled
        ? `${translation.get(FILTER_ARIA_KEYS[value])}; ${translation.get('library-availability')} — ${availabilityLabel}`
        : translation.get(FILTER_ARIA_KEYS[value]);
    return (
        <>
            <button
                ref={triggerRef}
                type="button"
                aria-label={ariaLabel}
                title={ariaLabel}
                onClick={() => setOpen((current) => !current)}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} text-gray-900 dark:text-white sm:gap-2 sm:justify-start w-10 sm:w-auto sm:px-4 sm:pl-6`}
            >
                <LuFilter
                    className={`sm:hidden w-5 h-5 ${active ? 'text-primary-500' : 'text-gray-600 dark:text-gray-300'}`}
                    aria-hidden="true"
                />
                <span className="hidden sm:inline whitespace-nowrap font-medium">
                    {label}
                </span>
                <LuChevronDown
                    className="hidden sm:block w-4 h-4 shrink-0 text-primary-400"
                    aria-hidden="true"
                />
            </button>
            <DropdownPortal
                triggerRef={triggerRef}
                open={open}
                onClose={() => setOpen(false)}
                role="dialog"
                aria-label={translation.get('filter.aria-label')}
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} w-56 max-h-[70vh] overflow-y-auto`}
            >
                <div
                    role="group"
                    aria-label={translation.get('library-reading-status')}
                    className="py-1"
                >
                    <p className="px-4 pt-2 pb-1 text-xs font-medium text-gray-500 dark:text-dark-400">
                        {translation.get('library-reading-status')}
                    </p>
                    {options.map((option) => (
                        <DropdownOption
                            key={option}
                            active={value === option}
                            aria-pressed={value === option}
                            className="flex items-center justify-between gap-3 text-sm"
                            onClick={() => {
                                onChange(option);
                                setOpen(false);
                            }}
                        >
                            {translation.get(FILTER_LABEL_KEYS[option])}
                            {value === option && (
                                <LuCheck
                                    className="w-4 h-4 shrink-0"
                                    aria-hidden="true"
                                />
                            )}
                        </DropdownOption>
                    ))}
                </div>
                {archiveEnabled && (
                    <div
                        role="group"
                        aria-label={translation.get('library-availability')}
                        className="border-t border-gray-200/50 dark:border-dark-700/50 py-1"
                    >
                        <p className="px-4 pt-2 pb-1 text-xs font-medium text-gray-500 dark:text-dark-400">
                            {translation.get('library-availability')}
                        </p>
                        {AVAILABILITY_OPTIONS.map((option) => (
                            <DropdownOption
                                key={option}
                                active={availability === option}
                                aria-pressed={availability === option}
                                className="flex items-center justify-between gap-3 text-sm"
                                onClick={() => {
                                    onAvailabilityChange(option);
                                    setOpen(false);
                                }}
                            >
                                {translation.get(
                                    AVAILABILITY_LABEL_KEYS[option],
                                )}
                                {availability === option && (
                                    <LuCheck
                                        className="w-4 h-4 shrink-0"
                                        aria-hidden="true"
                                    />
                                )}
                            </DropdownOption>
                        ))}
                    </div>
                )}
            </DropdownPortal>
        </>
    );
}
