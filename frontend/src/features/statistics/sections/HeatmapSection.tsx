import { useEffect, useLayoutEffect, useMemo, useRef } from 'react';

import {
    scrollToHorizontalOverflowRatio,
    scrollToHorizontalPosition,
} from '../../../shared/lib/dom/horizontal-scroll';
import {
    formatMonthOfYear,
    formatPlainDate,
} from '../../../shared/lib/intl/formatDate';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { DataFormatter } from '../lib/formatters';
import type { StatisticsYearResponse } from '../api/statistics-data';
import { translation } from '../../../shared/i18n';
import { TooltipManager } from '../../../shared/overlay/tooltip-manager';
import {
    HEATMAP_COLOR_CLASSES,
    calculateCellDate,
    formatISODate,
    normalizeHeatmapLevel,
} from '../model/heatmap';

const HEATMAP_ALL_COLOR_CLASSES = HEATMAP_COLOR_CLASSES.flat();

type HeatmapSectionProps = {
    selectedYear: number | null;
    yearData: StatisticsYearResponse | undefined;
    loading: boolean;
    animationSeed: string;
};

function prefersReducedMotion(): boolean {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function heatmapCellAnimationDelay(): number {
    return Math.floor(Math.random() * 340);
}

export function HeatmapSection({
    selectedYear,
    yearData,
    loading,
    animationSeed,
}: HeatmapSectionProps) {
    const scrollContainerRef = useRef<HTMLDivElement>(null);
    const heatmapContainerRef = useRef<HTMLDivElement>(null);
    const dayLabelsRef = useRef<HTMLDivElement>(null);
    const heatmapGridRef = useRef<HTMLDivElement>(null);
    const activityMap = useMemo(() => {
        const map = new Map<string, { pages: number; read: number }>();
        let maxActivity = 0;

        yearData?.daily_activity.forEach((entry) => {
            if (entry.reading_time_sec > maxActivity) {
                maxActivity = entry.reading_time_sec;
            }
            map.set(entry.date, {
                pages: entry.pages_read,
                read: entry.reading_time_sec,
            });
        });

        const configuredMax = yearData?.heatmap_config.max_scale_sec;
        if (configuredMax !== null && configuredMax !== undefined) {
            maxActivity = configuredMax;
        }

        return { map, maxActivity };
    }, [yearData]);

    const effectiveYear = yearData?.year ?? selectedYear;

    useEffect(() => {
        const scrollContainer = scrollContainerRef.current;
        const heatmapContainer = heatmapContainerRef.current;
        if (!scrollContainer || !heatmapContainer || !effectiveYear) {
            return;
        }

        if (effectiveYear === new Date().getFullYear()) {
            const weekWidth = heatmapContainer.scrollWidth / 53;
            const currentWeek = (() => {
                const today = new Date();
                const janFirst = new Date(today.getFullYear(), 0, 1);
                const janDayOfWeek = janFirst.getDay();
                const shiftToMonday =
                    janDayOfWeek === 0 ? -6 : 1 - janDayOfWeek;
                const firstMonday = new Date(janFirst);
                firstMonday.setDate(janFirst.getDate() + shiftToMonday);
                const daysDiff = Math.floor(
                    (today.getTime() - firstMonday.getTime()) /
                        (1000 * 60 * 60 * 24),
                );
                return Math.floor(daysDiff / 7);
            })();

            const targetPosition = currentWeek * weekWidth;
            scrollToHorizontalPosition(
                scrollContainer,
                heatmapContainer,
                targetPosition,
                0.7,
            );
            return;
        }

        scrollToHorizontalOverflowRatio(scrollContainer, heatmapContainer, 0.8);
    }, [effectiveYear, yearData]);

    useEffect(() => {
        const dayLabels = dayLabelsRef.current;
        const heatmapGrid = heatmapGridRef.current;
        if (!dayLabels || !heatmapGrid) {
            return;
        }

        const syncHeights = (): void => {
            dayLabels.style.height = `${heatmapGrid.offsetHeight}px`;
        };

        syncHeights();

        if (typeof ResizeObserver === 'undefined') {
            window.addEventListener('resize', syncHeights, { passive: true });
            return () => {
                window.removeEventListener('resize', syncHeights);
            };
        }

        const observer = new ResizeObserver(() => {
            syncHeights();
        });
        observer.observe(heatmapGrid);
        window.addEventListener('resize', syncHeights, { passive: true });

        return () => {
            observer.disconnect();
            window.removeEventListener('resize', syncHeights);
        };
    }, [effectiveYear, yearData]);

    const cellsByKey = useMemo(() => {
        const cellMap = new Map<
            string,
            { classes: string; tooltip: string; level: number }
        >();
        if (!effectiveYear) {
            return cellMap;
        }

        for (let week = 0; week < 53; week += 1) {
            for (let day = 0; day < 7; day += 1) {
                const date = calculateCellDate(effectiveYear, week, day);
                const dateIso = formatISODate(date);
                const activity = activityMap.map.get(dateIso) ?? {
                    pages: 0,
                    read: 0,
                };
                const level = normalizeHeatmapLevel(
                    activity.read,
                    activityMap.maxActivity,
                );
                const colorClasses = HEATMAP_COLOR_CLASSES[level].join(' ');
                const tooltipDate = formatPlainDate(dateIso, {
                    monthStyle: 'long',
                    yearDisplay: 'auto',
                });
                const tooltip = `${tooltipDate}: ${DataFormatter.formatReadTime(activity.read)}, ${DataFormatter.formatCount(activity.pages)} ${translation.get('pages-label', activity.pages)}`;

                cellMap.set(`${week}-${day}`, {
                    classes: colorClasses,
                    tooltip,
                    level,
                });
            }
        }

        return cellMap;
    }, [activityMap, effectiveYear]);

    useLayoutEffect(() => {
        const grid = heatmapGridRef.current;
        if (!grid || prefersReducedMotion()) {
            return;
        }

        const timeoutIds: number[] = [];
        const cells = grid.querySelectorAll<HTMLElement>('.activity-cell');

        cells.forEach((element) => {
            const weekStr = element.dataset.week;
            const dayStr = element.dataset.day;
            if (!weekStr || !dayStr) return;

            const cellData = cellsByKey.get(`${weekStr}-${dayStr}`);
            if (!cellData || cellData.level === 0) return;

            element.getAnimations().forEach((a) => a.cancel());
            HEATMAP_ALL_COLOR_CLASSES.forEach((c) =>
                element.classList.remove(c),
            );
            HEATMAP_COLOR_CLASSES[0].forEach((c) => element.classList.add(c));

            const level = cellData.level;
            const timeoutId = window.setTimeout(() => {
                if (!element.isConnected) return;

                HEATMAP_ALL_COLOR_CLASSES.forEach((c) =>
                    element.classList.remove(c),
                );
                HEATMAP_COLOR_CLASSES[level].forEach((c) =>
                    element.classList.add(c),
                );

                element.getAnimations().forEach((a) => a.cancel());
                element.animate(
                    [
                        {
                            transform: 'scale(0.62)',
                            filter: 'saturate(0.88) brightness(0.99)',
                        },
                        {
                            transform: 'scale(1.2)',
                            filter: 'saturate(1.22) brightness(1.04)',
                            offset: 0.52,
                        },
                        {
                            transform: 'scale(0.97)',
                            filter: 'saturate(1.03) brightness(1.01)',
                            offset: 0.78,
                        },
                        {
                            transform: 'scale(1)',
                            filter: 'saturate(1) brightness(1)',
                        },
                    ],
                    {
                        duration: 280,
                        easing: 'cubic-bezier(0.2, 0.9, 0.24, 1.1)',
                        fill: 'both',
                    },
                );
            }, heatmapCellAnimationDelay());

            timeoutIds.push(timeoutId);
        });

        return () => {
            timeoutIds.forEach((id) => window.clearTimeout(id));
        };
    }, [cellsByKey, animationSeed]);

    return (
        <div className="relative bg-white dark:bg-dark-850/50 rounded-lg p-3 sm:p-4 md:p-5 border border-gray-200/30 dark:border-dark-700/70">
            {loading && (
                <div className="absolute inset-0 z-10 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                    <LoadingSpinner
                        size="md"
                        srLabel="Loading reading heatmap"
                    />
                </div>
            )}

            <div className="flex">
                <div className="text-xs text-gray-500 dark:text-dark-400 font-medium w-8 sm:w-12 shrink-0 pr-2 sm:pr-4">
                    <div className="h-6 mb-3"></div>
                    <div
                        className="flex flex-col justify-between text-right"
                        id="dayLabels"
                        ref={dayLabelsRef}
                    >
                        <span>{translation.get('weekday.mon')}</span>
                        <span>{translation.get('weekday.thu')}</span>
                        <span>{translation.get('weekday.sun')}</span>
                    </div>
                </div>

                <div
                    className="flex-1 min-w-0 overflow-x-auto overflow-y-hidden scrollbar-hide"
                    id="heatmapScrollContainer"
                    ref={scrollContainerRef}
                >
                    <div
                        className="heatmap-container min-w-[680px] sm:min-w-[850px] md:min-w-[900px] lg:min-w-[1100px] xl:min-w-[1200px]"
                        id="readingHeatmap"
                        ref={heatmapContainerRef}
                    >
                        <div className="flex mb-3 mt-2 text-xs text-gray-500 dark:text-dark-400 font-medium">
                            <div className="flex w-full justify-between">
                                {Array.from({ length: 12 }, (_, monthIndex) => (
                                    <div
                                        key={monthIndex}
                                        className="w-8 text-center"
                                    >
                                        {formatMonthOfYear(monthIndex, {
                                            monthStyle: 'short',
                                        })}
                                    </div>
                                ))}
                            </div>
                        </div>

                        <div
                            key={effectiveYear ?? 'none'}
                            className="grid grid-cols-53 gap-1 w-full"
                            id="heatmapGrid"
                            ref={heatmapGridRef}
                        >
                            {Array.from({ length: 53 }, (_, week) => (
                                <div
                                    key={week}
                                    className="grid grid-rows-7 gap-1 sm:gap-1 md:gap-1 xl:gap-1.5"
                                >
                                    {Array.from({ length: 7 }, (_, day) => {
                                        const cell = cellsByKey.get(
                                            `${week}-${day}`,
                                        );
                                        const colorClasses =
                                            cell?.classes ??
                                            'bg-gray-100 dark:bg-dark-800';
                                        return (
                                            <div
                                                key={`${week}-${day}`}
                                                className={`w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-3.5 md:h-3.5 lg:w-4 lg:h-4 rounded-xs activity-cell hover:ring-1 hover:ring-inset hover:ring-gray-900 dark:hover:ring-white hover:z-10 ${colorClasses}`}
                                                data-week={week}
                                                data-day={day}
                                                ref={(element) => {
                                                    if (element && cell) {
                                                        TooltipManager.attach(
                                                            element,
                                                            cell.tooltip,
                                                        );
                                                    }
                                                }}
                                            ></div>
                                        );
                                    })}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>

            <div className="flex items-center justify-end mt-4 space-x-2 text-xs">
                <span className="text-gray-500 dark:text-dark-400">
                    {translation.get('less')}
                </span>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-4 md:h-4 rounded-xs bg-gray-100 dark:bg-dark-800"></div>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-4 md:h-4 rounded-xs bg-green-100 dark:bg-green-900"></div>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-4 md:h-4 rounded-xs bg-green-300 dark:bg-green-700"></div>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-4 md:h-4 rounded-xs bg-green-500 dark:bg-green-500"></div>
                <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 md:w-4 md:h-4 rounded-xs bg-green-600 dark:bg-green-300"></div>
                <span className="text-gray-500 dark:text-dark-400">
                    {translation.get('more')}
                </span>
            </div>
        </div>
    );
}
