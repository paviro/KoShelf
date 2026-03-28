import type { ReactNode } from 'react';
import type { IconType } from 'react-icons';

type FeaturedStatColor = 'blue' | 'purple';
type BlobPosition = 'bottom-right' | 'top-left';

type RecapFeaturedStatCardProps = {
    icon: IconType;
    value: ReactNode;
    label: ReactNode;
    color: FeaturedStatColor;
    blobPosition?: BlobPosition;
    className?: string;
};

const colorStyles: Record<
    FeaturedStatColor,
    {
        card: string;
        border: string;
        hoverBorder: string;
        blob: string;
        iconContainer: string;
        iconClass: string;
        labelClass: string;
    }
> = {
    blue: {
        card: 'bg-linear-to-br from-blue-500/10 via-blue-400/5 to-transparent dark:from-blue-500/20 dark:via-blue-400/10 dark:to-dark-800',
        border: 'border-blue-200/50 dark:border-blue-700/30',
        hoverBorder: 'hover:border-blue-300/70 dark:hover:border-blue-600/50',
        blob: 'bg-blue-400/20 dark:bg-blue-400/10',
        iconContainer:
            'bg-blue-500/20 dark:bg-linear-to-br dark:from-blue-500 dark:to-blue-600',
        iconClass: 'text-blue-600 dark:text-white',
        labelClass: 'text-blue-600 dark:text-blue-400',
    },
    purple: {
        card: 'bg-linear-to-br from-purple-500/10 via-purple-400/5 to-transparent dark:from-purple-500/20 dark:via-purple-400/10 dark:to-dark-800',
        border: 'border-purple-200/50 dark:border-purple-700/30',
        hoverBorder:
            'hover:border-purple-300/70 dark:hover:border-purple-600/50',
        blob: 'bg-purple-400/20 dark:bg-purple-400/10',
        iconContainer:
            'bg-purple-500/20 dark:bg-linear-to-br dark:from-purple-500 dark:to-purple-600',
        iconClass: 'text-purple-600 dark:text-white',
        labelClass: 'text-purple-600 dark:text-purple-400',
    },
};

const blobPositionStyles: Record<BlobPosition, string> = {
    'bottom-right': '-bottom-6 -right-6',
    'top-left': '-top-6 -left-6',
};

export function RecapFeaturedStatCard({
    icon: Icon,
    value,
    label,
    color,
    blobPosition = 'bottom-right',
    className,
}: RecapFeaturedStatCardProps) {
    const styles = colorStyles[color];
    const blobPos = blobPositionStyles[blobPosition];

    return (
        <div
            className={`${styles.card} border ${styles.border} rounded-xl p-4 shadow-xs relative overflow-hidden group hover:shadow-lg ${styles.hoverBorder} transition-all duration-300${className ? ` ${className}` : ''}`}
        >
            <div
                className={`absolute ${blobPos} w-16 h-16 ${styles.blob} rounded-full blur-xl group-hover:scale-150 transition-transform duration-500`}
            />
            <div className="flex items-center gap-3 relative z-10">
                <div
                    className={`w-10 h-10 rounded-xl ${styles.iconContainer} flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform duration-300`}
                >
                    <Icon
                        className={`w-5 h-5 ${styles.iconClass}`}
                        aria-hidden
                    />
                </div>
                <div className="flex flex-col min-w-0">
                    {value}
                    <span
                        className={`text-xs font-semibold ${styles.labelClass} uppercase tracking-wider`}
                    >
                        {label}
                    </span>
                </div>
            </div>
        </div>
    );
}
