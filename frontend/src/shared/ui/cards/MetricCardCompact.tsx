import type { ReactNode } from 'react';
import type { IconType } from 'react-icons';

type MetricCardCompactProps = {
    icon: IconType;
    iconContainerClassName: string;
    iconClassName: string;
    value: ReactNode;
    label: ReactNode;
    className?: string;
};

export function MetricCardCompact({
    icon: Icon,
    iconContainerClassName,
    iconClassName,
    value,
    label,
    className,
}: MetricCardCompactProps) {
    return (
        <div
            className={`bg-gray-50 dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/50 rounded-lg px-3 py-3 flex items-center${className ? ` ${className}` : ''}`}
        >
            <div
                className={`w-8 h-8 rounded-lg ${iconContainerClassName} flex items-center justify-center mr-2.5 shrink-0`}
            >
                <Icon
                    className={`w-4 h-4 ${iconClassName}`}
                    aria-hidden="true"
                />
            </div>
            <div>
                <div className="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                    {label}
                </div>
                <div className="text-base font-bold text-gray-900 dark:text-white leading-tight">
                    {value}
                </div>
            </div>
        </div>
    );
}
