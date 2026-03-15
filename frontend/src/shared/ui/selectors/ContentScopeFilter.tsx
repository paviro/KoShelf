import { useRef, useState } from 'react';
import { LuChevronDown, LuFilter } from 'react-icons/lu';

import type { ScopeValue } from '../../api';
import { useClickOutside } from '../../lib/dom/useClickOutside';
import { translation } from '../../i18n';
import {
    DROPDOWN_PANEL_BASE_CLASSNAME,
    DROPDOWN_TRIGGER_BASE_CLASSNAME,
} from '../dropdown/dropdown-styles';

type ContentScopeFilterProps = {
    visible: boolean;
    value: ScopeValue;
    onChange: (scope: ScopeValue) => void;
};

const OPTIONS: ScopeValue[] = ['all', 'books', 'comics'];

function scopeLabel(scope: ScopeValue): string {
    if (scope === 'books') {
        return translation.get('books');
    }

    if (scope === 'comics') {
        return translation.get('comics');
    }

    return translation.get('filter.all');
}

export function ContentScopeFilter({
    visible,
    value,
    onChange,
}: ContentScopeFilterProps) {
    const wrapperRef = useRef<HTMLDivElement>(null);
    const [open, setOpen] = useState(false);

    useClickOutside(wrapperRef, () => setOpen(false), open);

    if (!visible) {
        return null;
    }

    return (
        <div className="relative" ref={wrapperRef}>
            <button
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-label={translation.get('filter.aria-label')}
                onClick={() => setOpen((current) => !current)}
                className={`${DROPDOWN_TRIGGER_BASE_CLASSNAME} text-gray-900 dark:text-white sm:gap-2 sm:justify-start w-10 sm:w-auto sm:px-4 sm:pl-6`}
            >
                <LuFilter
                    className={`sm:hidden w-5 h-5 ${value === 'all' ? 'text-gray-600 dark:text-gray-300' : 'text-primary-500'}`}
                    aria-hidden="true"
                />
                <span className="hidden sm:inline font-medium">
                    {scopeLabel(value)}
                </span>
                <LuChevronDown
                    className="hidden sm:block w-4 h-4 text-primary-400"
                    aria-hidden="true"
                />
            </button>

            <div
                className={`${DROPDOWN_PANEL_BASE_CLASSNAME} right-0 w-40 ${open ? '' : 'hidden'}`}
                role="menu"
            >
                {OPTIONS.map((scope) => {
                    const active = scope === value;

                    return (
                        <button
                            key={scope}
                            type="button"
                            className={`block w-full text-left px-4 py-2 hover:bg-gray-100/50 dark:hover:bg-dark-700/50 ${
                                active
                                    ? 'text-primary-700 dark:text-primary-300 font-medium'
                                    : 'text-gray-700 dark:text-dark-200'
                            }`}
                            onClick={() => {
                                onChange(scope);
                                setOpen(false);
                            }}
                        >
                            {scopeLabel(scope)}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
