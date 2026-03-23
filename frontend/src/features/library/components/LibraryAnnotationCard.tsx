import { useEffect, useRef, useState } from 'react';
import { BsBookmarkFill } from 'react-icons/bs';
import {
    LuBookOpen,
    LuCheck,
    LuClock3,
    LuFileText,
    LuHash,
    LuNotebookPen,
    LuPencil,
    LuTrash2,
} from 'react-icons/lu';
import { Link } from 'react-router';

import { translation } from '../../../shared/i18n';
import { formatAnnotationDatetime } from '../lib/library-detail-formatters';
import type { LibraryAnnotation } from '../api/library-data';
import { HighlightColorPicker } from './HighlightColorPicker';
import { HighlightDrawerPicker, DRAWER_ICONS } from './HighlightDrawerPicker';

type LibraryAnnotationCardVariant = 'highlight' | 'bookmark';

type LibraryAnnotationCardProps = {
    annotation: LibraryAnnotation;
    variant: LibraryAnnotationCardVariant;
    readerHref?: string | null;
    canWrite?: boolean;
    onSaveNote?: (note: string | null) => void;
    onColorChange?: (color: string) => void;
    onDrawerChange?: (drawer: string) => void;
    onDelete?: () => void;
};

const VARIANT_STYLES: Record<
    LibraryAnnotationCardVariant,
    {
        noteLabelClass: string;
        leadingBadgeClass: string;
    }
> = {
    highlight: {
        noteLabelClass: 'text-primary-400',
        leadingBadgeClass: 'text-amber-500',
    },
    bookmark: {
        noteLabelClass: 'text-primary-400',
        leadingBadgeClass: 'text-yellow-500',
    },
};

const BOOKMARK_QUOTE_BAR = 'from-yellow-400 to-yellow-600';

const HIGHLIGHT_QUOTE_BAR: Record<string, string> = {
    yellow: 'from-amber-400 to-amber-600',
    orange: 'from-orange-400 to-orange-600',
    red: 'from-red-400 to-red-600',
    green: 'from-emerald-400 to-emerald-600',
    olive: 'from-lime-400 to-lime-600',
    blue: 'from-blue-400 to-blue-600',
    cyan: 'from-cyan-400 to-cyan-600',
    purple: 'from-purple-400 to-purple-600',
    gray: 'from-gray-400 to-gray-600',
};

const HIGHLIGHT_COLOR_CSS: Record<string, string> = {
    yellow: 'bg-yellow-400',
    green: 'bg-emerald-400',
    blue: 'bg-blue-400',
    red: 'bg-red-400',
    orange: 'bg-orange-400',
    olive: 'bg-lime-500',
    cyan: 'bg-cyan-400',
    purple: 'bg-purple-400',
    gray: 'bg-gray-400',
};

export function LibraryAnnotationCard({
    annotation,
    variant,
    readerHref,
    canWrite = false,
    onSaveNote,
    onColorChange,
    onDrawerChange,
    onDelete,
}: LibraryAnnotationCardProps) {
    const styles = VARIANT_STYLES[variant];
    const formattedDate = formatAnnotationDatetime(annotation.datetime);
    const hasText = annotation.text !== null && annotation.text !== undefined;
    const [confirmingDelete, setConfirmingDelete] = useState(false);
    const [colorPickerOpen, setColorPickerOpen] = useState(false);
    const colorButtonRef = useRef<HTMLButtonElement>(null);
    const [drawerPickerOpen, setDrawerPickerOpen] = useState(false);
    const drawerButtonRef = useRef<HTMLButtonElement>(null);

    // Inline note editing (draft state only — optimistic display is handled
    // at the React Query cache level by the mutation hooks).
    const [editingNote, setEditingNote] = useState(false);
    const [draftNote, setDraftNote] = useState(annotation.note ?? '');
    const textareaRef = useRef<HTMLTextAreaElement>(null);
    const suppressToolbarTransition = useRef(false);

    const hasNote = annotation.note !== null && annotation.note !== undefined;
    const hasBody = hasText || hasNote;

    useEffect(() => {
        if (editingNote) {
            setDraftNote(annotation.note ?? '');
            // Focus on next frame after render
            requestAnimationFrame(() => {
                if (textareaRef.current) {
                    const ta = textareaRef.current;
                    const len = ta.value.length;
                    ta.focus();
                    ta.setSelectionRange(len, len);
                }
            });
        }
    }, [editingNote, annotation.note]);

    const colorDotClass =
        HIGHLIGHT_COLOR_CSS[annotation.color ?? 'yellow'] ?? 'bg-yellow-400';

    const quoteBarClass =
        variant === 'bookmark'
            ? BOOKMARK_QUOTE_BAR
            : (HIGHLIGHT_QUOTE_BAR[annotation.color ?? 'yellow'] ??
              HIGHLIGHT_QUOTE_BAR.yellow);

    const DrawerIcon = DRAWER_ICONS[annotation.drawer ?? 'lighten'] ?? DRAWER_ICONS.lighten;

    const hasWriteCapabilities =
        canWrite && (onSaveNote || onColorChange || onDrawerChange || onDelete);
    const showToolbar = hasWriteCapabilities && !editingNote;

    const stopEditingNote = () => {
        suppressToolbarTransition.current = true;
        setEditingNote(false);
        requestAnimationFrame(() => {
            suppressToolbarTransition.current = false;
        });
    };

    const handleNoteSave = () => {
        const trimmed = draftNote.trim();
        onSaveNote?.(trimmed || null);
        stopEditingNote();
    };

    return (
        <article className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg overflow-hidden shadow-xs">
            {/* Header — metadata only */}
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
                        <span className="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs bg-gray-200/50 dark:bg-dark-700/50 text-gray-500 dark:text-dark-400">
                            <LuClock3
                                className="w-3.5 h-3.5"
                                aria-hidden="true"
                            />
                            {formattedDate}
                        </span>
                    )}

                    {readerHref && (
                        <Link
                            to={readerHref}
                            title={translation.get('open-at-annotation')}
                            aria-label={translation.get('open-at-annotation')}
                            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-primary-600 dark:text-primary-300 bg-primary-500/10 hover:bg-primary-500/20 border border-primary-500/20 hover:border-primary-500/30 transition-colors"
                        >
                            <LuBookOpen
                                className="w-3.5 h-3.5"
                                aria-hidden="true"
                            />
                            <span className="hidden sm:inline">
                                {translation.get('open-in-reader')}
                            </span>
                        </Link>
                    )}
                </div>
            </header>

            {/* Body */}
            {(hasBody || editingNote) && (
                <div className="p-6">
                    {hasText && (
                        <div className="relative">
                            <div
                                className={`absolute top-0 left-0 w-1 h-full bg-linear-to-b ${quoteBarClass} rounded-full`}
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

                    {(hasNote || editingNote) && (
                        <div className={hasText ? 'mt-6' : ''}>
                            <div className="flex items-center mb-3">
                                <div className="h-px bg-gray-200 dark:bg-dark-700 grow mr-3"></div>
                                <div className="flex items-center space-x-2">
                                    <div className="w-6 h-6 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-full flex items-center justify-center">
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
                                <div className="h-px bg-gray-200 dark:bg-dark-700 grow ml-3"></div>
                            </div>
                            {editingNote ? (
                                <textarea
                                    ref={textareaRef}
                                    value={draftNote}
                                    onChange={(e) => setDraftNote(e.target.value)}
                                    rows={3}
                                    className="w-full rounded-lg border border-gray-200 dark:border-dark-700 bg-gray-50 dark:bg-dark-800 text-gray-900 dark:text-white p-3 text-sm leading-relaxed focus:ring-2 focus:ring-primary-500/30 focus:border-primary-500 resize-y placeholder:text-gray-400 dark:placeholder:text-dark-500"
                                    placeholder={translation.get('add-note')}
                                />
                            ) : (
                                <div className="bg-gray-100 dark:bg-dark-850/50 p-4 rounded-lg border border-gray-200 dark:border-dark-700/30">
                                    <p className="text-gray-700 dark:text-dark-200 leading-relaxed whitespace-pre-wrap">
                                        {annotation.note}
                                    </p>
                                </div>
                            )}
                        </div>
                    )}
                </div>
            )}

            {/* Note editing footer */}
            {editingNote && (
                <div className="flex items-center justify-between px-6 py-3 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30">
                    {hasNote ? (
                        <button
                            type="button"
                            onClick={() => {
                                onSaveNote?.(null);
                                stopEditingNote();
                            }}
                            className="inline-flex items-center gap-1.5 px-4 py-2 text-sm text-red-500 dark:text-red-400 border border-red-300/50 dark:border-red-500/30 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
                        >
                            <LuTrash2 className="w-3.5 h-3.5" aria-hidden="true" />
                            {translation.get('delete-note')}
                        </button>
                    ) : (
                        <div />
                    )}
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={stopEditingNote}
                            className="px-4 py-2 text-sm text-gray-500 dark:text-dark-400 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                        >
                            {translation.get('cancel')}
                        </button>
                        <button
                            type="button"
                            onClick={handleNoteSave}
                            className="px-4 py-2 text-sm font-medium text-primary-600 dark:text-primary-400 border border-primary-500/30 dark:border-primary-500/20 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-500/10 transition-colors"
                        >
                            {translation.get('save')}
                        </button>
                    </div>
                </div>
            )}

            {/* Footer — compact action bar, animated on section toggle only */}
            <div
                className={`grid ${!editingNote && !suppressToolbarTransition.current ? 'transition-[grid-template-rows] duration-200 ease-in-out' : ''}`}
                style={{
                    gridTemplateRows: showToolbar ? '1fr' : '0fr',
                }}
            >
                <div className="overflow-hidden min-h-0">
                    <footer className="flex items-center justify-between px-4 py-1.5 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30">
                        {onDelete ? (
                            <div className="flex items-center">
                                {confirmingDelete ? (
                                    <div className="inline-flex items-center gap-0.5">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                onDelete();
                                                setConfirmingDelete(false);
                                            }}
                                            className="inline-flex items-center gap-1.5 px-2 py-1.5 text-sm font-medium text-red-500 dark:text-red-400 rounded-md hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
                                        >
                                            <LuCheck
                                                className="w-3.5 h-3.5"
                                                aria-hidden="true"
                                            />
                                            {translation.get('delete')}
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() =>
                                                setConfirmingDelete(false)
                                            }
                                            className="px-2 py-1.5 text-sm text-gray-500 dark:text-dark-400 rounded-md hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                                        >
                                            {translation.get('cancel')}
                                        </button>
                                    </div>
                                ) : (
                                    <button
                                        type="button"
                                        onClick={() => setConfirmingDelete(true)}
                                        className="inline-flex items-center gap-1.5 px-2 py-1.5 text-sm text-red-500 dark:text-red-400 rounded-md hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
                                    >
                                        <LuTrash2
                                            className="w-3.5 h-3.5"
                                            aria-hidden="true"
                                        />
                                        {variant === 'bookmark'
                                            ? translation.get('delete-bookmark')
                                            : hasNote
                                              ? translation.get('delete-highlight-and-note')
                                              : translation.get('delete-highlight')}
                                    </button>
                                )}
                            </div>
                        ) : (
                            <div />
                        )}

                        <div className="flex items-center gap-0.5">
                            {variant === 'highlight' && onDrawerChange && (
                                <>
                                    <button
                                        ref={drawerButtonRef}
                                        type="button"
                                        onClick={() => {
                                            setColorPickerOpen(false);
                                            setDrawerPickerOpen(!drawerPickerOpen);
                                        }}
                                        className="inline-flex items-center p-1.5 rounded-md text-gray-500 dark:text-dark-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                                        title={translation.get('highlight-drawer.aria-label')}
                                        aria-label={translation.get('highlight-drawer.aria-label')}
                                    >
                                        <DrawerIcon className="w-3.5 h-3.5" aria-hidden="true" />
                                    </button>
                                    {drawerPickerOpen && (
                                        <HighlightDrawerPicker
                                            anchorRef={drawerButtonRef}
                                            currentDrawer={annotation.drawer ?? 'lighten'}
                                            onSelect={(drawer) => {
                                                onDrawerChange(drawer);
                                                setDrawerPickerOpen(false);
                                            }}
                                            onClose={() => setDrawerPickerOpen(false)}
                                        />
                                    )}
                                </>
                            )}

                            {variant === 'highlight' && onColorChange && (
                                <>
                                    <button
                                        ref={colorButtonRef}
                                        type="button"
                                        onClick={() => {
                                            setDrawerPickerOpen(false);
                                            setColorPickerOpen(!colorPickerOpen);
                                        }}
                                        className="inline-flex items-center p-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                                        title={translation.get('highlight-color.aria-label')}
                                        aria-label={translation.get(
                                            'highlight-color.aria-label',
                                        )}
                                    >
                                        <span
                                            className={`w-3.5 h-3.5 rounded-full ${colorDotClass} border border-black/10 dark:border-white/20`}
                                        />
                                    </button>
                                    {colorPickerOpen && (
                                        <HighlightColorPicker
                                            anchorRef={colorButtonRef}
                                            currentColor={
                                                annotation.color ?? 'yellow'
                                            }
                                            onSelect={(color) => {
                                                onColorChange(color);
                                                setColorPickerOpen(false);
                                            }}
                                            onClose={() =>
                                                setColorPickerOpen(false)
                                            }
                                        />
                                    )}
                                </>
                            )}

                            {onSaveNote && (
                                <button
                                    type="button"
                                    onClick={() => setEditingNote(!editingNote)}
                                    className="inline-flex items-center p-1.5 rounded-md text-gray-500 dark:text-dark-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors"
                                    title={
                                        hasNote
                                            ? translation.get('edit-note')
                                            : translation.get('add-note')
                                    }
                                    aria-label={
                                        hasNote
                                            ? translation.get('edit-note')
                                            : translation.get('add-note')
                                    }
                                >
                                    <LuPencil
                                        className="w-3.5 h-3.5"
                                        aria-hidden="true"
                                    />
                                </button>
                            )}
                        </div>
                    </footer>
                </div>
            </div>
        </article>
    );
}
