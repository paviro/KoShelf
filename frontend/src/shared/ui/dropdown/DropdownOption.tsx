import type { ButtonHTMLAttributes, ReactNode } from 'react';

type DropdownOptionProps = {
    active?: boolean;
    separator?: boolean;
    children: ReactNode;
} & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'type'>;

export function DropdownOption({
    active = false,
    separator = false,
    className = '',
    children,
    ...rest
}: DropdownOptionProps) {
    return (
        <button
            type="button"
            className={`block w-full text-left px-4 py-2 cursor-pointer hover:bg-gray-100/50 dark:hover:bg-dark-700/50 transition-colors duration-200 ${
                separator
                    ? 'border-b border-gray-200/30 dark:border-dark-700/30'
                    : ''
            } ${
                active
                    ? 'text-primary-700 dark:text-primary-300 font-medium'
                    : 'text-gray-700 dark:text-dark-200'
            } ${className}`}
            {...rest}
        >
            {children}
        </button>
    );
}
