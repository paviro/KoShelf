import { useMemo } from 'react';

import { TooltipManager } from '../../../shared/overlay/tooltip-manager';

export type DistributionBarItem = {
    readTime: number;
    tooltip: string;
    label: string;
};

type DistributionBarChartProps = {
    items: DistributionBarItem[];
    columns: number;
    heightClassName: string;
    barClassName: string;
};

export function DistributionBarChart({
    items,
    columns,
    heightClassName,
    barClassName,
}: DistributionBarChartProps) {
    const maxReadTime = useMemo(
        () => Math.max(...items.map((item) => item.readTime), 0),
        [items],
    );

    return (
        <div
            className={`${heightClassName} grid gap-2 sm:gap-3 items-end`}
            style={{
                gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
            }}
        >
            {items.map((item, index) => {
                let heightPercent = 2;
                if (maxReadTime > 0 && item.readTime > 0) {
                    heightPercent = Math.max(
                        (item.readTime / maxReadTime) * 100,
                        8,
                    );
                }

                return (
                    <div
                        key={index}
                        className="h-full flex flex-col justify-end"
                    >
                        <div className="relative h-full flex items-end">
                            <div
                                className={`w-full cursor-pointer rounded-t-sm bg-linear-to-t opacity-35 transition-[height,opacity] duration-500 ease-out overflow-hidden ${barClassName}`}
                                style={{
                                    height: `${heightPercent}%`,
                                    opacity: item.readTime > 0 ? 1 : 0.35,
                                }}
                                data-tooltip-gap="5"
                                aria-label={item.tooltip}
                                ref={(element) => {
                                    if (element) {
                                        TooltipManager.attach(
                                            element,
                                            item.tooltip,
                                        );
                                    }
                                }}
                            >
                                <span className="block h-[2px] w-full bg-white/75 dark:bg-white/45"></span>
                            </div>
                        </div>
                        <div className="mt-3 text-center text-xs text-gray-500 dark:text-dark-400 leading-none">
                            {item.label}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
