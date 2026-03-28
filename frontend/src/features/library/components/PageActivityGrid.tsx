import { useEffect, useLayoutEffect, useMemo, useRef } from 'react';

import { translation } from '../../../shared/i18n';
import { useDarkMode } from '../../../shared/lib/dom/useDarkMode';
import { formatDuration } from '../../../shared/lib/intl/formatDuration';
import { TooltipManager } from '../../../shared/overlay/tooltip-manager';
import type {
    ChapterEntry,
    PageActivityAnnotation,
} from '../../../shared/contracts';
import {
    type AggregatedPage,
    buildAnnotationMap,
    percentileDuration,
} from '../lib/page-activity-data';

// ── Continuous color scale (teal → blue → violet) ────────────────────────

/** Compute an oklch color string for a bar given its 0–1 intensity ratio. */
function barColor(ratio: number, dark: boolean): string {
    // Hue: 230 (blue) → 280 (indigo) — stays cool, no pink
    const hue = 230 + ratio * 50;
    const chroma = dark ? 0.04 + ratio * 0.23 : 0.02 + ratio * 0.13;
    // Lightness: wider range for more contrast
    const lightness = dark
        ? 0.3 + ratio * 0.34 // very dim → bright
        : 0.87 - ratio * 0.27; // very pale → moderate
    return `oklch(${lightness} ${chroma} ${hue})`;
}

/** Base color shown before animation reveals the real color. */
function barBaseColor(dark: boolean): string {
    return dark ? 'oklch(0.25 0.01 230)' : 'oklch(0.92 0.01 230)';
}

// ── Annotation dot colors ────────────────────────────────────────────────

type AnnotationDots = {
    highlight: string | null;
    bookmark: string | null;
    note: string | null;
};

function annotationDots(
    annotations: PageActivityAnnotation[],
    dark: boolean,
): AnnotationDots {
    return {
        highlight: annotations.some((a) => a.kind === 'highlight')
            ? dark
                ? 'bg-yellow-400'
                : 'bg-yellow-500'
            : null,
        bookmark: annotations.some((a) => a.kind === 'bookmark')
            ? dark
                ? 'bg-rose-400'
                : 'bg-rose-500'
            : null,
        note: annotations.some((a) => a.kind === 'note')
            ? dark
                ? 'bg-fuchsia-400'
                : 'bg-fuchsia-500'
            : null,
    };
}

// ── Animation ───────────────────────────────────────────────────────────

function prefersReducedMotion(): boolean {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

// ── Height calculation ──────────────────────────────────────────────────

/** Minimum height % for unread pages (thin baseline). */
const MIN_BAR_PCT = 3;
/** Minimum height % for read pages so they're always visible. */
const MIN_READ_BAR_PCT = 8;
/** Minimum width in px per bar — triggers horizontal scrolling below this. */
const MIN_BAR_WIDTH_PX = 2;

function barHeightPercent(duration: number, maxDuration: number): number {
    if (duration <= 0 || maxDuration <= 0) return MIN_BAR_PCT;
    // Use sqrt scaling to spread values more evenly.
    const ratio = Math.sqrt(duration / maxDuration);
    return MIN_READ_BAR_PCT + ratio * (100 - MIN_READ_BAR_PCT);
}

// ── Component ───────────────────────────────────────────────────────────

/** Alternating background tint classes for chapter bands. */
const CHAPTER_BAND_CLASSES = ['', 'bg-gray-500/[0.04] dark:bg-white/[0.03]'];

/** Alternating color classes for the bottom chapter strip segments. */
const CHAPTER_STRIP_CLASSES = [
    'bg-gray-300/60 dark:bg-dark-600/60',
    'bg-gray-400/50 dark:bg-dark-500/50',
];

type ChapterRange = {
    title: string;
    startPage: number;
    endPage: number;
    /** Width as a percentage of totalPages. */
    widthPct: number;
    /** Index for alternating colors. */
    bandIndex: number;
};

type PageActivityGridProps = {
    totalPages: number;
    pageData: Map<number, AggregatedPage>;
    annotations: PageActivityAnnotation[];
    chapters: ChapterEntry[];
    animationSeed: string;
};

export function PageActivityGrid({
    totalPages,
    pageData,
    annotations,
    chapters,
    animationSeed,
}: PageActivityGridProps) {
    const dark = useDarkMode();
    const containerRef = useRef<HTMLDivElement>(null);
    const annotationMap = useMemo(
        () => buildAnnotationMap(annotations),
        [annotations],
    );

    // Convert fractional chapter positions to page numbers.
    const chaptersWithPages = useMemo(
        () =>
            chapters.map((ch) => ({
                title: ch.title,
                page: Math.round(
                    Math.min(1, Math.max(0, ch.position)) * totalPages,
                ),
            })),
        [chapters, totalPages],
    );

    // Build page → chapter title lookup for chapter boundary gaps.
    const chapterMap = useMemo(() => {
        const map = new Map<number, string>();
        for (const ch of chaptersWithPages) {
            if (ch.page >= 1 && ch.page <= totalPages) {
                map.set(ch.page, ch.title);
            }
        }
        return map;
    }, [chaptersWithPages, totalPages]);

    // Build sorted chapter ranges for background bands and bottom strip.
    const chapterRanges = useMemo((): ChapterRange[] => {
        if (chaptersWithPages.length === 0) return [];
        const sorted = [...chaptersWithPages]
            .filter((ch) => ch.page >= 1 && ch.page <= totalPages)
            .sort((a, b) => a.page - b.page);
        if (sorted.length === 0) return [];

        const ranges: ChapterRange[] = [];
        for (let i = 0; i < sorted.length; i++) {
            const startPage = sorted[i].page;
            const endPage =
                i + 1 < sorted.length ? sorted[i + 1].page - 1 : totalPages;
            ranges.push({
                title: sorted[i].title,
                startPage,
                endPage,
                widthPct: ((endPage - startPage + 1) / totalPages) * 100,
                bandIndex: i % 2,
            });
        }
        return ranges;
    }, [chaptersWithPages, totalPages]);

    // Raw max for height scaling (outliers keep their tall bars).
    const maxDuration = useMemo(() => {
        let max = 0;
        for (const p of pageData.values()) {
            if (p.totalDuration > max) max = p.totalDuration;
        }
        return max;
    }, [pageData]);

    // P90 max for color levels — outliers don't crush everything to level 1.
    const colorMax = useMemo(
        () => percentileDuration(pageData, 90),
        [pageData],
    );

    // Pre-compute bar data.
    const bars = useMemo(() => {
        const result: Array<{
            page: number;
            heightPct: number;
            color: string;
            tooltip: string;
            dots: AnnotationDots;
            isRead: boolean;
        }> = [];

        for (let page = 1; page <= totalPages; page++) {
            const data = pageData.get(page);
            const isRead = Boolean(data);
            const colorRatio = data
                ? Math.min(Math.sqrt(data.totalDuration / (colorMax || 1)), 1)
                : 0;
            const heightPct = data
                ? barHeightPercent(data.totalDuration, maxDuration)
                : 0;
            const color = isRead ? barColor(colorRatio, dark) : 'transparent';
            const pageAnnotations = annotationMap.get(page);
            const dots: AnnotationDots = pageAnnotations
                ? annotationDots(pageAnnotations, dark)
                : { highlight: null, bookmark: null, note: null };

            const chapterTitle = chapterMap.get(page);

            let tooltip: string;
            if (data) {
                const duration = formatDuration(data.totalDuration, {
                    includeSeconds: true,
                });
                const visits = data.readCount;
                tooltip = `${translation.get('page-activity.page', { page })} — ${duration}, ${translation.get('page-activity.visits', visits)}`;
            } else {
                tooltip = `${translation.get('page-activity.page', { page })} — ${translation.get('page-activity.unread')}`;
            }

            if (pageAnnotations) {
                const highlightCount = pageAnnotations.filter(
                    (a) => a.kind === 'highlight',
                ).length;
                const bookmarkCount = pageAnnotations.filter(
                    (a) => a.kind === 'bookmark',
                ).length;
                const noteCount = pageAnnotations.filter(
                    (a) => a.kind === 'note',
                ).length;
                const parts: string[] = [];
                if (highlightCount > 0)
                    parts.push(
                        translation.get(
                            'page-activity.highlights',
                            highlightCount,
                        ),
                    );
                if (noteCount > 0)
                    parts.push(
                        translation.get('page-activity.notes', noteCount),
                    );
                if (bookmarkCount > 0)
                    parts.push(
                        translation.get(
                            'page-activity.bookmarks',
                            bookmarkCount,
                        ),
                    );
                if (parts.length > 0) tooltip += ` (${parts.join(', ')})`;
            }

            if (chapterTitle) {
                tooltip = `${chapterTitle}\n${tooltip}`;
            }

            result.push({
                page,
                heightPct,
                color,
                tooltip,
                dots,
                isRead,
            });
        }

        return result;
    }, [
        totalPages,
        pageData,
        maxDuration,
        colorMax,
        annotationMap,
        chapterMap,
        dark,
    ]);

    // Animation seed tracked via ref to keep it out of the layout effect deps
    // (avoids a race with placeholder data cancelling in-flight animations).
    const animationSeedRef = useRef(animationSeed);
    const mountAnimateRef = useRef(true);
    const prevAnimationSeedRef = useRef(animationSeed);

    useLayoutEffect(() => {
        animationSeedRef.current = animationSeed;
    });

    useEffect(() => {
        return () => {
            mountAnimateRef.current = true;
        };
    }, []);

    useLayoutEffect(() => {
        const container = containerRef.current;
        if (!container) return;

        const currentSeed = animationSeedRef.current;
        const seedChanged = prevAnimationSeedRef.current !== currentSeed;
        const shouldAnimate = mountAnimateRef.current || seedChanged;
        mountAnimateRef.current = false;
        prevAnimationSeedRef.current = currentSeed;

        const barElements =
            container.querySelectorAll<HTMLElement>('.page-activity-bar');

        // Data refetch or theme change: update bar styles instantly.
        if (!shouldAnimate) {
            barElements.forEach((element) => {
                const pageStr = element.dataset.page;
                if (!pageStr) return;
                const bar = bars[Number(pageStr) - 1];
                if (!bar || !bar.isRead) return;
                element.getAnimations().forEach((a) => a.cancel());
                element.style.height = `${bar.heightPct}%`;
                element.style.backgroundColor = bar.color;
            });
            return;
        }

        // Seed changed (mount or completion filter change): animate bars growing up.
        if (prefersReducedMotion()) return;

        const timeoutIds: number[] = [];
        const base = barBaseColor(dark);

        barElements.forEach((element) => {
            const pageStr = element.dataset.page;
            if (!pageStr) return;
            const page = Number(pageStr);
            const bar = bars[page - 1];
            if (!bar || !bar.isRead) return;

            // Start bars at baseline with muted color.
            element.style.height = `${MIN_BAR_PCT}%`;
            element.style.backgroundColor = base;

            const targetHeight = bar.heightPct;

            // Sequential reveal: bars animate left-to-right with a small stagger.
            const delay = (page / totalPages) * 600;
            const timeoutId = window.setTimeout(() => {
                if (!element.isConnected) return;

                element.getAnimations().forEach((a) => a.cancel());
                element.animate(
                    [
                        { height: `${MIN_BAR_PCT}%`, backgroundColor: base },
                        {
                            height: `${targetHeight}%`,
                            backgroundColor: bar.color,
                        },
                    ],
                    {
                        duration: 350,
                        easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)',
                        fill: 'forwards',
                    },
                );
            }, delay);

            timeoutIds.push(timeoutId);
        });

        return () => {
            timeoutIds.forEach((id) => window.clearTimeout(id));
        };
    }, [bars, totalPages, dark]);

    const pagesRead = pageData.size;
    const readPercent =
        totalPages > 0 ? Math.round((pagesRead / totalPages) * 100) : 0;

    const hasChapters = chapterRanges.length > 0;

    // Only show legend items for annotation types actually present.
    const hasHighlights = annotations.some((a) => a.kind === 'highlight');
    const hasBookmarks = annotations.some((a) => a.kind === 'bookmark');
    const hasNotes = annotations.some((a) => a.kind === 'note');

    const minWidthPx = totalPages * MIN_BAR_WIDTH_PX;

    const hasAnnotationLegend = hasHighlights || hasBookmarks || hasNotes;

    return (
        <div className="bg-white dark:bg-dark-850/50 rounded-lg p-3 sm:p-4 md:p-5 border border-gray-200/30 dark:border-dark-700/70">
            {/* Scrollable wrapper — kicks in when bars would be < 2px */}
            <div className="overflow-x-auto overflow-y-hidden scrollbar-hide">
                <div style={{ minWidth: `${minWidthPx}px` }}>
                    {/* Annotation strip along the top */}
                    <div className="flex h-1 overflow-hidden mb-px bg-gray-200/40 dark:bg-dark-700/40">
                        {bars.map((bar) => {
                            const kinds: { cls: string; key: string }[] = [];
                            if (bar.dots.bookmark)
                                kinds.push({
                                    cls: bar.dots.bookmark,
                                    key: 'b',
                                });
                            if (bar.dots.note)
                                kinds.push({ cls: bar.dots.note, key: 'n' });
                            if (bar.dots.highlight)
                                kinds.push({
                                    cls: bar.dots.highlight,
                                    key: 'h',
                                });
                            if (kinds.length === 0)
                                return (
                                    <div
                                        key={bar.page}
                                        className="flex-1 min-w-0"
                                    />
                                );
                            return (
                                <div
                                    key={bar.page}
                                    className="flex-1 min-w-0 flex flex-col"
                                >
                                    {kinds.map((k) => (
                                        <div
                                            key={k.key}
                                            className={`w-full flex-1 ${k.cls}`}
                                        />
                                    ))}
                                </div>
                            );
                        })}
                    </div>

                    {/* Bar chart container */}
                    <div
                        ref={containerRef}
                        className="relative flex items-end"
                        style={{ height: '200px' }}
                    >
                        {/* Alternating chapter background bands */}
                        {hasChapters && (
                            <div className="absolute inset-0 flex pointer-events-none">
                                {chapterRanges.map((range) => (
                                    <div
                                        key={range.startPage}
                                        className={
                                            CHAPTER_BAND_CLASSES[
                                                range.bandIndex
                                            ]
                                        }
                                        style={{ width: `${range.widthPct}%` }}
                                    />
                                ))}
                            </div>
                        )}

                        {/* Bars */}
                        {bars.map((bar) => (
                            <div
                                key={bar.page}
                                className="relative flex-1 min-w-0 flex flex-col items-center justify-end h-full"
                                ref={(element) => {
                                    if (element) {
                                        TooltipManager.attach(
                                            element,
                                            bar.tooltip,
                                        );
                                    }
                                }}
                            >
                                <div
                                    data-page={bar.page}
                                    className="page-activity-bar w-full rounded-t-[1px] transition-colors hover:brightness-110 flex flex-col"
                                    style={{
                                        height: `${bar.heightPct}%`,
                                        backgroundColor: bar.color,
                                    }}
                                >
                                    {(bar.dots.bookmark || bar.dots.note) && (
                                        <div className="w-full shrink-0 flex flex-col rounded-t-[1px]">
                                            {bar.dots.bookmark && (
                                                <div
                                                    className={`w-full ${bar.dots.bookmark}`}
                                                    style={{ height: '6px' }}
                                                />
                                            )}
                                            {bar.dots.note && (
                                                <div
                                                    className={`w-full ${bar.dots.note}`}
                                                    style={{ height: '6px' }}
                                                />
                                            )}
                                        </div>
                                    )}
                                    {bar.dots.highlight && (
                                        <div
                                            className={`w-full shrink-0 rounded-t-[1px] ${bar.dots.highlight}`}
                                            style={{ height: '6px' }}
                                        />
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>

                    {/* Chapter strip along the bottom */}
                    {hasChapters && (
                        <div className="flex h-1.5 overflow-hidden">
                            {chapterRanges.map((range) => (
                                <div
                                    key={range.startPage}
                                    className={
                                        CHAPTER_STRIP_CLASSES[range.bandIndex]
                                    }
                                    style={{ width: `${range.widthPct}%` }}
                                    title={range.title}
                                />
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {/* Footer: summary + legends */}
            <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 mt-4 text-xs font-medium">
                <span className="text-gray-500 dark:text-dark-400">
                    {pagesRead}{' '}
                    {translation.get('page-activity.of', { total: totalPages })}{' '}
                    {translation.get('page-activity.pages-read')}{' '}
                    <span className="text-gray-400 dark:text-dark-500">
                        ({readPercent}%)
                    </span>
                </span>

                {(hasAnnotationLegend || hasChapters) && (
                    <div className="flex items-center gap-3 text-gray-500 dark:text-dark-400">
                        {hasHighlights && (
                            <span className="flex items-center gap-1.5">
                                <span className="inline-block w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-xs bg-yellow-400" />
                                {translation.get('highlights-label', 1)}
                            </span>
                        )}
                        {hasBookmarks && (
                            <span className="flex items-center gap-1.5">
                                <span className="inline-block w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-xs bg-rose-400" />
                                {translation.get(
                                    'page-activity.legend-bookmark',
                                )}
                            </span>
                        )}
                        {hasNotes && (
                            <span className="flex items-center gap-1.5">
                                <span className="inline-block w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-xs bg-fuchsia-400" />
                                {translation.get('notes-label', 1)}
                            </span>
                        )}
                        {hasChapters && (
                            <span className="flex items-center gap-1.5">
                                <span className="inline-flex w-2.5 sm:w-3 gap-px">
                                    <span className="inline-block flex-1 h-2.5 sm:h-3 rounded-xs bg-gray-300 dark:bg-dark-600" />
                                    <span className="inline-block flex-1 h-2.5 sm:h-3 rounded-xs bg-gray-400 dark:bg-dark-500" />
                                </span>
                                {translation.get(
                                    'page-activity.legend-chapter',
                                )}
                            </span>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
