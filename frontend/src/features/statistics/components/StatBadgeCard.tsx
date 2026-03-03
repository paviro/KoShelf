import type { ReactNode } from 'react';

type StatBadgeCardVariant = 'responsive' | 'compact';

type StatBadgeCardProps = {
    iconPath: string;
    iconContainerClassName: string;
    iconClassName: string;
    value: ReactNode;
    label: ReactNode;
    valueId?: string;
    variant?: StatBadgeCardVariant;
};

export function StatBadgeCard({
    iconPath,
    iconContainerClassName,
    iconClassName,
    value,
    label,
    valueId,
    variant = 'responsive',
}: StatBadgeCardProps) {
    if (variant === 'compact') {
        return (
            <div className="@container bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
                <div className="flex items-center gap-3">
                    <div
                        className={`w-10 h-10 rounded-lg ${iconContainerClassName} flex items-center justify-center flex-shrink-0`}
                    >
                        <svg
                            className={`w-5 h-5 ${iconClassName}`}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d={iconPath}
                            ></path>
                        </svg>
                    </div>
                    <div>
                        <div
                            id={valueId}
                            className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white"
                        >
                            {value}
                        </div>
                        <div className="text-sm text-gray-500 dark:text-dark-400">{label}</div>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="@container bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-3 sm:p-4">
            <div className="flex flex-col @[140px]:flex-row items-center @[140px]:items-center gap-2 @[140px]:gap-3 h-full">
                <div
                    className={`w-10 h-10 rounded-lg ${iconContainerClassName} flex items-center justify-center flex-shrink-0`}
                >
                    <svg
                        className={`w-5 h-5 ${iconClassName}`}
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            d={iconPath}
                        ></path>
                    </svg>
                </div>
                <div className="text-center @[140px]:text-left">
                    <div
                        id={valueId}
                        className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white"
                    >
                        {value}
                    </div>
                    <div className="text-sm text-gray-500 dark:text-dark-400">{label}</div>
                </div>
            </div>
        </div>
    );
}
