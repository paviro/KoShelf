import type { ReactNode } from 'react';
import type { IconType } from 'react-icons';

type MetricCardVariant = 'responsive' | 'compact';
type MetricCardSize = 'default' | 'sm';

type MetricCardProps = {
    icon: IconType;
    iconContainerClassName: string;
    iconClassName: string;
    value: ReactNode;
    label: ReactNode;
    valueId?: string;
    variant?: MetricCardVariant;
    size?: MetricCardSize;
    className?: string;
};

export function MetricCard({
    icon: Icon,
    iconContainerClassName,
    iconClassName,
    value,
    label,
    valueId,
    variant = 'responsive',
    size = 'default',
    className,
}: MetricCardProps) {
    const sm = size === 'sm';
    const valueClass = sm
        ? 'text-lg font-bold text-gray-900 dark:text-white'
        : 'text-xl md:text-2xl font-bold text-gray-900 dark:text-white';
    const labelClass = sm
        ? '-mt-0.5 text-xs font-medium text-gray-500 dark:text-dark-400'
        : '-mt-0.5 text-sm font-medium text-gray-500 dark:text-dark-400';
    const paddingClass = sm ? 'p-3' : 'p-3 sm:p-4';

    const layoutClass =
        variant === 'compact'
            ? 'flex items-center gap-3'
            : sm
              ? 'flex flex-col @[120px]:flex-row items-center gap-2 @[120px]:gap-3 h-full'
              : 'flex flex-col @[140px]:flex-row items-center gap-2 @[140px]:gap-3 h-full';

    const textAlignClass =
        variant === 'compact'
            ? ''
            : sm
              ? 'text-center @[120px]:text-left'
              : 'text-center @[140px]:text-left';

    return (
        <div
            className={`@container bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg ${paddingClass}${className ? ` ${className}` : ''}`}
        >
            <div className={layoutClass}>
                <div
                    className={`w-10 h-10 rounded-lg ${iconContainerClassName} flex items-center justify-center shrink-0`}
                >
                    <Icon
                        className={`w-5 h-5 ${iconClassName}`}
                        aria-hidden="true"
                    />
                </div>
                <div className={textAlignClass || undefined}>
                    <div id={valueId} className={valueClass}>
                        {value}
                    </div>
                    <div className={labelClass}>{label}</div>
                </div>
            </div>
        </div>
    );
}
