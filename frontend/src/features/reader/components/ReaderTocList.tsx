import { useEffect, useMemo, useRef } from 'react';
import { LuFileText } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import {
    normalizeReaderText,
    resolveSectionCandidates,
} from '../lib/reader-drawer-utils';
import type { TocEntry } from '../model/reader-model';
import { ReaderDrawerEmptyState } from './ReaderDrawerEmptyState';

type ReaderTocListProps = {
    toc: TocEntry[];
    currentChapter: string;
    currentChapterHref: string | null;
    currentSectionIndex: number | null;
    onSelect: (href: string) => void;
};

function normalizeHref(value: string): string {
    return value.trim().replace(/^\.\//, '').toLowerCase();
}

function stripFragment(value: string): string {
    const hashIndex = value.indexOf('#');
    return hashIndex >= 0 ? value.slice(0, hashIndex) : value;
}

function hrefsMatch(hrefA: string, hrefB: string): boolean {
    const normalizedA = normalizeHref(hrefA);
    const normalizedB = normalizeHref(hrefB);
    if (normalizedA === '' || normalizedB === '') {
        return false;
    }

    if (normalizedA === normalizedB) {
        return true;
    }

    return stripFragment(normalizedA) === stripFragment(normalizedB);
}

function labelsMatch(entryLabel: string, normalizedCurrent: string): boolean {
    if (normalizedCurrent === '') {
        return false;
    }

    const normalizedEntry = normalizeReaderText(entryLabel);
    return (
        normalizedEntry === normalizedCurrent ||
        normalizedEntry.startsWith(normalizedCurrent) ||
        normalizedCurrent.startsWith(normalizedEntry)
    );
}

function resolveCurrentIndexBySection(
    toc: TocEntry[],
    currentSectionIndex: number | null,
): number {
    const candidates = resolveSectionCandidates(currentSectionIndex);
    for (const candidate of candidates) {
        let bestMatchIndex = -1;
        let bestMatchSection = -1;

        for (let i = 0; i < toc.length; i += 1) {
            const sectionIndex = toc[i].sectionIndex;
            if (typeof sectionIndex !== 'number') {
                continue;
            }

            if (sectionIndex <= candidate && sectionIndex >= bestMatchSection) {
                bestMatchSection = sectionIndex;
                bestMatchIndex = i;
            }
        }

        if (bestMatchIndex >= 0) {
            return bestMatchIndex;
        }
    }

    return -1;
}

export function ReaderTocList({
    toc,
    currentChapter,
    currentChapterHref,
    currentSectionIndex,
    onSelect,
}: ReaderTocListProps) {
    const listRef = useRef<HTMLDivElement>(null);
    const normalizedCurrent = normalizeReaderText(currentChapter);

    const resolvedCurrentIndex = useMemo(
        () =>
            toc.findIndex((entry) => {
                if (
                    currentChapterHref &&
                    hrefsMatch(entry.href, currentChapterHref)
                ) {
                    return true;
                }

                return labelsMatch(entry.label, normalizedCurrent);
            }),
        [currentChapterHref, normalizedCurrent, toc],
    );

    const sectionFallbackIndex = useMemo(
        () => resolveCurrentIndexBySection(toc, currentSectionIndex),
        [currentSectionIndex, toc],
    );

    const currentIndex =
        resolvedCurrentIndex >= 0 ? resolvedCurrentIndex : sectionFallbackIndex;

    useEffect(() => {
        if (toc.length === 0 || currentIndex < 0 || !listRef.current) {
            return;
        }

        const scrollContainer = listRef.current.closest<HTMLElement>(
            '[data-tabbed-drawer-scroll-container]',
        );
        if (scrollContainer) {
            scrollContainer.style.overflowY = 'hidden';
        }

        const restoreOverflow = () => {
            if (scrollContainer) {
                scrollContainer.style.overflowY = '';
            }
        };

        const scroll = () => {
            const currentItem = listRef.current?.querySelector<HTMLElement>(
                '[data-current-toc-item]',
            );
            if (!currentItem) {
                return;
            }

            currentItem.scrollIntoView({
                block: 'center',
                inline: 'nearest',
            });
        };

        const frameId = requestAnimationFrame(scroll);
        const timeoutId = window.setTimeout(scroll, 350);
        const restoreOverflowId = window.setTimeout(restoreOverflow, 425);

        return () => {
            cancelAnimationFrame(frameId);
            window.clearTimeout(timeoutId);
            window.clearTimeout(restoreOverflowId);
            restoreOverflow();
        };
    }, [currentIndex, toc.length]);

    if (toc.length === 0) {
        return (
            <ReaderDrawerEmptyState
                icon={LuFileText}
                variant="contents"
                title={translation.get('reader-no-toc')}
                description={translation.get('reader-no-toc.description')}
            />
        );
    }

    return (
        <div ref={listRef} className="flex flex-col gap-0.5 py-2">
            {toc.map((entry, index) => {
                const isCurrent = index === currentIndex;
                const isSubsection = entry.depth > 0;
                const hasTopLevelSpacing = entry.depth === 0 && index > 0;
                const inactiveToneClass = isSubsection
                    ? 'text-gray-500 dark:text-dark-400 hover:text-gray-700 dark:hover:text-dark-200'
                    : 'text-gray-700 dark:text-dark-200 hover:text-gray-900 dark:hover:text-white';
                const activeToneClass = isSubsection
                    ? 'text-gray-800 dark:text-dark-200'
                    : 'text-gray-900 dark:text-white';
                const activeBackdropClass = isSubsection
                    ? 'after:bg-gray-100 dark:after:bg-dark-700/50'
                    : 'after:bg-gray-100 dark:after:bg-dark-700/65';
                const labelClass = isSubsection
                    ? `text-[0.925rem] leading-snug ${
                          isCurrent ? 'font-medium' : 'font-normal'
                      }`
                    : 'text-[0.925rem] leading-snug font-semibold';

                return (
                    <button
                        key={`${entry.href}-${index}`}
                        type="button"
                        onClick={() => onSelect(entry.href)}
                        data-current-toc-item={isCurrent || undefined}
                        style={{
                            paddingInlineStart: `${
                                0.75 + Math.min(entry.depth, 6) * 1.2
                            }rem`,
                        }}
                        className={`relative isolate flex items-center gap-3 text-left px-2 py-2 rounded-lg transition-colors duration-150 cursor-pointer ${
                            isCurrent ? activeToneClass : inactiveToneClass
                        } ${
                            isCurrent
                                ? `after:content-[''] after:absolute after:inset-x-0 after:-inset-y-0.5 after:rounded-xl after:-z-10 ${activeBackdropClass}`
                                : ''
                        } ${hasTopLevelSpacing ? 'mt-1.5' : ''}`}
                    >
                        <span
                            className={`w-1 self-stretch rounded-full shrink-0 ${
                                isCurrent
                                    ? 'bg-amber-500 dark:bg-amber-300'
                                    : 'bg-transparent'
                            }`}
                        />
                        <span className={labelClass}>{entry.label}</span>
                    </button>
                );
            })}
        </div>
    );
}
