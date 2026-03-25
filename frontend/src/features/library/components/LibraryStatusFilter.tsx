import { useMemo } from 'react';

import { translation } from '../../../shared/i18n';
import {
    FilterDropdown,
    type FilterDropdownOption,
} from '../../../shared/ui/selectors/FilterDropdown';
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

export function LibraryStatusFilter({
    value,
    options,
    onChange,
}: LibraryStatusFilterProps) {
    const dropdownOptions = useMemo<FilterDropdownOption<LibraryFilterValue>[]>(
        () =>
            options.map((o) => ({
                value: o,
                label: translation.get(FILTER_LABEL_KEYS[o]),
            })),
        [options],
    );

    return (
        <FilterDropdown
            value={value}
            options={dropdownOptions}
            onChange={onChange}
            ariaLabel={translation.get(FILTER_ARIA_KEYS[value])}
            panelClassName="w-max min-w-40"
            separateOptions
            optionClassName="text-sm md:text-base whitespace-nowrap"
        />
    );
}
