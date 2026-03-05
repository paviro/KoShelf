import { BsBookmarkFill } from 'react-icons/bs';
import { LuClock3, LuFileText, LuHash, LuNotebookPen } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { formatAnnotationDatetime } from '../lib/library-detail-formatters';
import type { LibraryAnnotation } from '../api/library-data';

type LibraryAnnotationCardVariant = 'highlight' | 'bookmark';

type LibraryAnnotationCardProps = {
    annotation: LibraryAnnotation;
    variant: LibraryAnnotationCardVariant;
};

const VARIANT_STYLES: Record<
    LibraryAnnotationCardVariant,
    {
        quoteBarClass: string;
        noteLabelClass: string;
        leadingBadgeClass: string;
    }
> = {
    highlight: {
        quoteBarClass: 'from-amber-400 to-amber-600',
        noteLabelClass: 'text-primary-400',
        leadingBadgeClass: 'text-amber-500',
    },
    bookmark: {
        quoteBarClass: 'from-yellow-400 to-yellow-600',
        noteLabelClass: 'text-primary-400',
        leadingBadgeClass: 'text-yellow-500',
    },
};

export function LibraryAnnotationCard({
    annotation,
    variant,
}: LibraryAnnotationCardProps) {
    const styles = VARIANT_STYLES[variant];
    const formattedDate = formatAnnotationDatetime(annotation.datetime);
    const hasText = annotation.text !== null && annotation.text !== undefined;
    const hasNote = annotation.note !== null && annotation.note !== undefined;
    const hasBody = hasText || hasNote;

    return (
        <article className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg overflow-hidden shadow-sm">
            <header className="flex items-center justify-between text-sm text-gray-500 dark:text-dark-400 px-6 py-3 bg-gray-100/50 dark:bg-dark-850/50 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center gap-3">
                    {variant === 'bookmark' && (
                        <span
                            className={`inline-flex items-center ${styles.leadingBadgeClass}`}
                        >
                            <BsBookmarkFill
                                className="w-4 h-4 mr-1"
                                aria-hidden="true"
                            />
                            {translation.get('page-bookmark')}
                        </span>
                    )}

                    {annotation.chapter && (
                        <span className="inline-flex items-center">
                            <LuFileText
                                className="w-4 h-4 mr-1 text-primary-400"
                                aria-hidden="true"
                            />
                            {annotation.chapter}
                        </span>
                    )}

                    {typeof annotation.pageno === 'number' && (
                        <span className="hidden sm:inline-flex items-center">
                            <LuHash
                                className="w-4 h-4 mr-1 text-primary-400"
                                aria-hidden="true"
                            />
                            {translation.get('page-number', annotation.pageno)}
                        </span>
                    )}
                </div>

                <div className="flex items-center gap-3">
                    {typeof annotation.pageno === 'number' && (
                        <span className="sm:hidden inline-flex items-center">
                            <LuHash
                                className="w-4 h-4 mr-1 text-primary-400"
                                aria-hidden="true"
                            />
                            {translation.get('page-number', annotation.pageno)}
                        </span>
                    )}

                    {formattedDate && (
                        <span className="hidden sm:inline-flex items-center">
                            <LuClock3
                                className="w-4 h-4 mr-1 text-primary-400"
                                aria-hidden="true"
                            />
                            {formattedDate}
                        </span>
                    )}
                </div>
            </header>

            {hasBody && (
                <div className="p-6">
                    {hasText && (
                        <div className="relative">
                            <div
                                className={`absolute top-0 left-0 w-1 h-full bg-gradient-to-b ${styles.quoteBarClass} rounded-full`}
                            ></div>

                            {variant === 'bookmark' && (
                                <div className="pl-6 mb-1">
                                    <span className="text-sm text-yellow-600 dark:text-yellow-300 uppercase tracking-wider font-semibold">
                                        {translation.get('bookmark-anchor')}:
                                    </span>
                                </div>
                            )}

                            <blockquote className="text-gray-900 dark:text-white text-lg leading-relaxed pl-6 font-light whitespace-pre-wrap">
                                {annotation.text}
                            </blockquote>
                        </div>
                    )}

                    {hasNote && (
                        <div className={hasText ? 'mt-6' : ''}>
                            <div className="flex items-center mb-3">
                                <div className="h-px bg-gray-200 dark:bg-dark-700 flex-grow mr-3"></div>
                                <div className="flex items-center space-x-2">
                                    <div className="w-6 h-6 bg-primary-500/20 dark:bg-gradient-to-br dark:from-primary-500 dark:to-primary-600 rounded-full flex items-center justify-center">
                                        <LuNotebookPen
                                            className="w-3 h-3 text-primary-600 dark:text-white"
                                            aria-hidden="true"
                                        />
                                    </div>
                                    <div
                                        className={`text-sm font-medium ${styles.noteLabelClass} uppercase tracking-wider`}
                                    >
                                        {translation.get('my-note')}
                                    </div>
                                </div>
                                <div className="h-px bg-gray-200 dark:bg-dark-700 flex-grow ml-3"></div>
                            </div>
                            <div className="bg-gray-100 dark:bg-dark-850/50 p-4 rounded-lg border border-gray-200 dark:border-dark-700/30">
                                <p className="text-gray-700 dark:text-dark-200 leading-relaxed whitespace-pre-wrap">
                                    {annotation.note}
                                </p>
                            </div>
                        </div>
                    )}
                </div>
            )}
        </article>
    );
}
