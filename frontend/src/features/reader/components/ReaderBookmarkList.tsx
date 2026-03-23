import { useMemo, useRef } from 'react';
import { BsBookmarkFill } from 'react-icons/bs';

import { translation } from '../../../shared/i18n';
import type { LibraryAnnotation } from '../../library/api/library-data';
import { useDrawerListScroll } from '../hooks/useDrawerListScroll';
import {
    groupAnnotationsByChapter,
    normalizeReaderText,
    resolveCurrentGroupIndex,
} from '../lib/reader-drawer-utils';
import { ReaderDrawerEmptyState } from './ReaderDrawerEmptyState';

type ReaderBookmarkListProps = {
    bookmarks: LibraryAnnotation[];
    currentChapter: string;
    currentSectionIndex: number | null;
    onSelect: (annotation: LibraryAnnotation) => void;
};

export function ReaderBookmarkList({
    bookmarks,
    currentChapter,
    currentSectionIndex,
    onSelect,
}: ReaderBookmarkListProps) {
    const groups = useMemo(
        () => groupAnnotationsByChapter(bookmarks),
        [bookmarks],
    );
    const listRef = useRef<HTMLDivElement>(null);

    const normalizedCurrent = normalizeReaderText(currentChapter);
    const currentGroupIndex = useMemo(
        () =>
            resolveCurrentGroupIndex(
                groups,
                normalizedCurrent,
                currentSectionIndex,
            ),
        [currentSectionIndex, groups, normalizedCurrent],
    );

    useDrawerListScroll(listRef, currentGroupIndex, 'data-current-chapter');

    if (bookmarks.length === 0) {
        return (
            <ReaderDrawerEmptyState
                icon={BsBookmarkFill}
                variant="bookmarks"
                title={translation.get('reader-no-bookmarks')}
                description={translation.get('reader-no-bookmarks.description')}
            />
        );
    }

    return (
        <div ref={listRef} className="flex flex-col gap-4 py-2">
            {groups.map((group, groupIndex) => {
                const isCurrent = groupIndex === currentGroupIndex;

                return (
                    <div
                        key={groupIndex}
                        data-current-chapter={isCurrent || undefined}
                    >
                        {group.chapter && (
                            <p
                                className={`text-xs font-semibold uppercase tracking-wide px-2 pb-1.5 ${
                                    isCurrent
                                        ? 'text-amber-700 dark:text-amber-300'
                                        : 'text-gray-400 dark:text-dark-400'
                                }`}
                            >
                                {group.chapter}
                            </p>
                        )}
                        <div className="flex flex-col gap-0.5">
                            {group.annotations.map((annotation, index) => (
                                <button
                                    key={index}
                                    type="button"
                                    onClick={() => onSelect(annotation)}
                                    className="cursor-pointer flex items-start gap-3 text-left px-2 py-2.5 rounded-lg transition-colors duration-150 hover:bg-gray-100 dark:hover:bg-dark-700/50"
                                >
                                    <span className="w-5 h-5 rounded-md shrink-0 mt-0.5 bg-yellow-100 dark:bg-yellow-500/20 text-yellow-600 dark:text-yellow-300 flex items-center justify-center">
                                        <BsBookmarkFill
                                            className="w-2.5 h-2.5"
                                            aria-hidden="true"
                                        />
                                    </span>
                                    <div className="min-w-0 flex-1">
                                        {annotation.text ? (
                                            <p className="text-[0.925rem] text-gray-700 dark:text-dark-200 line-clamp-3 leading-snug">
                                                {annotation.text}
                                            </p>
                                        ) : (
                                            <p className="text-[0.925rem] font-medium text-gray-500 dark:text-dark-300 leading-snug">
                                                {translation.get(
                                                    'page-bookmark',
                                                )}
                                            </p>
                                        )}
                                        {typeof annotation.pageno ===
                                            'number' && (
                                            <p className="mt-0.5 text-xs font-medium text-gray-400 dark:text-dark-400">
                                                {translation.get(
                                                    'page-number',
                                                    annotation.pageno,
                                                )}
                                            </p>
                                        )}
                                    </div>
                                </button>
                            ))}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
