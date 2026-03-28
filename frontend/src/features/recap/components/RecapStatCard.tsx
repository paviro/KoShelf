import type { ReactNode } from 'react';
import type { IconType } from 'react-icons';

type RecapStatCardColor = 'red' | 'amber' | 'orange' | 'pink';

type RecapStatCardProps = {
    icon: IconType;
    value: ReactNode;
    label: ReactNode;
    color: RecapStatCardColor;
    className?: string;
};

const colorStyles: Record<
    RecapStatCardColor,
    { hoverBorder: string; iconContainer: string; iconClass: string }
> = {
    red: {
        hoverBorder: 'hover:border-red-300/50 dark:hover:border-red-700/40',
        iconContainer:
            'bg-red-500/20 dark:bg-linear-to-br dark:from-red-500 dark:to-orange-500',
        iconClass: 'text-red-500 dark:text-white',
    },
    amber: {
        hoverBorder:
            'hover:border-amber-300/50 dark:hover:border-amber-700/40',
        iconContainer:
            'bg-amber-500/20 dark:bg-linear-to-br dark:from-amber-500 dark:to-yellow-500',
        iconClass: 'text-amber-500 dark:text-white',
    },
    orange: {
        hoverBorder:
            'hover:border-orange-300/50 dark:hover:border-orange-700/40',
        iconContainer:
            'bg-orange-500/20 dark:bg-linear-to-br dark:from-orange-500 dark:to-amber-500',
        iconClass: 'text-orange-500 dark:text-white',
    },
    pink: {
        hoverBorder: 'hover:border-pink-300/50 dark:hover:border-pink-700/40',
        iconContainer:
            'bg-pink-500/20 dark:bg-linear-to-br dark:from-pink-500 dark:to-rose-500',
        iconClass: 'text-pink-500 dark:text-white',
    },
};

export function RecapStatCard({
    icon: Icon,
    value,
    label,
    color,
    className,
}: RecapStatCardProps) {
    const styles = colorStyles[color];

    return (
        <div
            className={`h-full bg-white dark:bg-dark-800/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl px-3 py-3 shadow-xs hover:shadow-md ${styles.hoverBorder} transition-all duration-300 group flex items-center${className ? ` ${className}` : ''}`}
        >
            <div className="flex items-center gap-2.5">
                <div
                    className={`w-8 h-8 rounded-lg ${styles.iconContainer} flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform duration-300`}
                >
                    <Icon
                        className={`w-4 h-4 ${styles.iconClass}`}
                        aria-hidden
                    />
                </div>
                <div className="flex flex-col min-w-0">
                    {value}
                    <span className="text-[11px] font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                        {label}
                    </span>
                </div>
            </div>
        </div>
    );
}
