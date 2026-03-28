import {
    type ReactNode,
    type RefObject,
    useEffect,
    useRef,
    useState,
} from 'react';
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
import { Button } from '../../../shared/ui/button/Button';
import { formatAnnotationDatetime } from '../lib/library-detail-formatters';
import {
    colorDotClass,
    colorQuoteBarGradient,
    DRAWER_ICONS,
} from '../lib/highlight-constants';
import type { LibraryAnnotation } from '../api/library-data';
import { HighlightColorPicker } from './HighlightColorPicker';
import { HighlightDrawerPicker } from './HighlightDrawerPicker';

type LibraryAnnotationCardVariant = 'highlight' | 'bookmark';

type LibraryAnnotationCardProps = {
    annotation: LibraryAnnotation;
    variant: LibraryAnnotationCardVariant;
    readerHref?: string | null;
    showEditingControls?: boolean;
    onSaveNote?: (note: string | null) => void;
    onColorChange?: (color: string) => void;
    onDrawerChange?: (drawer: string) => void;
    onDelete?: () => void;
};

/* ------------------------------------------------------------------ */
/*  Sub-components (file-local)                                       */
/* ------------------------------------------------------------------ */

function CardHeader({
    variant,
    chapter,
    pageno,
    formattedDate,
    readerHref,
}: {
    variant: LibraryAnnotationCardVariant;
    chapter?: string | null;
    pageno?: number | null;
    formattedDate: string | null;
    readerHref?: string | null;
}) {
    const isBookmark = variant === 'bookmark';
    const Tag = isBookmark ? 'div' : 'header';

    return (
        <Tag
            className={`flex items-center justify-between text-sm font-medium text-gray-500 dark:text-dark-400 ${isBookmark ? 'px-5' : 'px-6'} py-3${isBookmark ? '' : ' bg-gray-100/50 dark:bg-dark-850/50 border-b border-gray-200/50 dark:border-dark-700/50'}`}
        >
            <div className="flex items-center gap-3 min-w-0">
                {chapter && (
                    <span className="inline-flex items-center min-w-0">
                        <LuFileText
                            className={`w-4 h-4 ${isBookmark ? 'mr-1.5' : 'mr-1'} text-primary-400 shrink-0`}
                            aria-hidden="true"
                        />
                        <span className="truncate">{chapter}</span>
                    </span>
                )}

                {typeof pageno === 'number' && (
                    <span
                        className={`hidden sm:inline-flex items-center${isBookmark ? ' shrink-0' : ''}`}
                    >
                        <LuHash
                            className="w-4 h-4 mr-1 text-primary-400"
                            aria-hidden="true"
                        />
                        {translation.get('page-number', pageno)}
                    </span>
                )}
            </div>

            <div
                className={`flex items-center ${isBookmark ? 'gap-2 shrink-0 ml-3' : 'gap-3'}`}
            >
                {typeof pageno === 'number' && (
                    <span className="sm:hidden inline-flex items-center">
                        <LuHash
                            className="w-4 h-4 mr-1 text-primary-400"
                            aria-hidden="true"
                        />
                        {pageno}
                    </span>
                )}

                {formattedDate && (
                    <span className="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs bg-gray-200/50 dark:bg-dark-700/50 text-gray-500 dark:text-dark-400">
                        <LuClock3 className="w-3.5 h-3.5" aria-hidden="true" />
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
        </Tag>
    );
}

function NoteEditor({
    variant,
    note,
    editingNote,
    draftNote,
    onDraftChange,
    textareaRef,
    hasText,
}: {
    variant: LibraryAnnotationCardVariant;
    note?: string | null;
    editingNote: boolean;
    draftNote: string;
    onDraftChange: (value: string) => void;
    textareaRef: RefObject<HTMLTextAreaElement | null>;
    hasText: boolean;
}) {
    const isHighlight = variant === 'highlight';

    return (
        <div className={isHighlight && hasText ? 'mt-6' : ''}>
            {isHighlight && (
                <div className="flex items-center mb-3">
                    <div className="h-px bg-gray-200 dark:bg-dark-700 grow mr-3"></div>
                    <div className="flex items-center space-x-2">
                        <div className="w-6 h-6 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-full flex items-center justify-center">
                            <LuNotebookPen
                                className="w-3 h-3 text-primary-600 dark:text-white"
                                aria-hidden="true"
                            />
                        </div>
                        <div className="text-sm font-medium text-primary-400 uppercase tracking-wider">
                            {translation.get('my-note')}
                        </div>
                    </div>
                    <div className="h-px bg-gray-200 dark:bg-dark-700 grow ml-3"></div>
                </div>
            )}
            {editingNote ? (
                <textarea
                    ref={textareaRef}
                    value={draftNote}
                    onChange={(e) => onDraftChange(e.target.value)}
                    rows={3}
                    className="w-full rounded-lg border border-gray-200 dark:border-dark-700 bg-gray-50 dark:bg-dark-800 text-gray-900 dark:text-white p-3 text-sm leading-relaxed focus:ring-2 focus:ring-primary-500/30 focus:border-primary-500 resize-y placeholder:text-gray-400 dark:placeholder:text-dark-500"
                    placeholder={translation.get('add-note')}
                />
            ) : (
                <div
                    className={`bg-gray-100 dark:bg-dark-850/50 ${isHighlight ? 'p-4' : 'p-3'} rounded-lg border border-gray-200 dark:border-dark-700/30`}
                >
                    <p
                        className={`${isHighlight ? '' : 'text-sm '}text-gray-700 dark:text-dark-200 leading-relaxed whitespace-pre-wrap`}
                    >
                        {note}
                    </p>
                </div>
            )}
        </div>
    );
}

function NoteEditingFooter({
    px,
    hasNote,
    onDeleteNote,
    onCancel,
    onSave,
}: {
    px: string;
    hasNote: boolean;
    onDeleteNote: () => void;
    onCancel: () => void;
    onSave: () => void;
}) {
    return (
        <div
            className={`flex items-center justify-between ${px} py-3 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30`}
        >
            {hasNote ? (
                <Button color="danger" icon={LuTrash2} onClick={onDeleteNote}>
                    {translation.get('delete-note')}
                </Button>
            ) : (
                <div />
            )}
            <div className="flex items-center gap-2">
                <Button color="secondary" onClick={onCancel}>
                    {translation.get('cancel')}
                </Button>
                <Button onClick={onSave}>{translation.get('save')}</Button>
            </div>
        </div>
    );
}

function DeleteConfirmation({
    label,
    confirming,
    onConfirm,
    onStartConfirm,
    onCancelConfirm,
}: {
    label: string;
    confirming: boolean;
    onConfirm: () => void;
    onStartConfirm: () => void;
    onCancelConfirm: () => void;
}) {
    if (confirming) {
        return (
            <div className="inline-flex items-center gap-0.5">
                <Button
                    variant="ghost"
                    color="danger"
                    size="xs"
                    icon={LuCheck}
                    onClick={onConfirm}
                >
                    {translation.get('delete')}
                </Button>
                <Button variant="ghost" size="xs" onClick={onCancelConfirm}>
                    {translation.get('cancel')}
                </Button>
            </div>
        );
    }

    return (
        <Button
            variant="ghost"
            color="danger"
            size="xs"
            icon={LuTrash2}
            onClick={onStartConfirm}
        >
            {label}
        </Button>
    );
}

function AnimatedToolbar({
    visible,
    editingNote,
    suppressTransition,
    children,
}: {
    visible: boolean;
    editingNote: boolean;
    suppressTransition: boolean;
    children: ReactNode;
}) {
    return (
        <div
            className={`grid ${!editingNote && !suppressTransition ? 'transition-[grid-template-rows] duration-200 ease-in-out' : ''}`}
            style={{ gridTemplateRows: visible ? '1fr' : '0fr' }}
        >
            <div className="overflow-hidden min-h-0">
                <footer className="flex items-center justify-between px-4 py-1.5 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30">
                    {children}
                </footer>
            </div>
        </div>
    );
}

/* ------------------------------------------------------------------ */
/*  Main component                                                    */
/* ------------------------------------------------------------------ */

export function LibraryAnnotationCard({
    annotation,
    variant,
    readerHref,
    showEditingControls = false,
    onSaveNote,
    onColorChange,
    onDrawerChange,
    onDelete,
}: LibraryAnnotationCardProps) {
    const isBookmark = variant === 'bookmark';
    const isHighlight = variant === 'highlight';
    const px = isBookmark ? 'px-5' : 'px-6';

    const formattedDate = formatAnnotationDatetime(annotation.datetime);
    const hasText = annotation.text != null;
    const hasNote = annotation.note != null;
    const hasBody = hasText || hasNote;

    const [confirmingDelete, setConfirmingDelete] = useState(false);
    const [activePicker, setActivePicker] = useState<'color' | 'drawer' | null>(
        null,
    );
    const colorButtonRef = useRef<HTMLButtonElement>(null);
    const drawerButtonRef = useRef<HTMLButtonElement>(null);

    const [editingNote, setEditingNote] = useState(false);
    const [draftNote, setDraftNote] = useState(annotation.note ?? '');
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    const [suppressToolbarTransition, setSuppressToolbarTransition] =
        useState(false);

    const [prevShowEditingControls, setPrevShowEditingControls] =
        useState(showEditingControls);
    const [prevEditingNote, setPrevEditingNote] = useState(false);
    const [prevAnnotationNote, setPrevAnnotationNote] = useState(
        annotation.note,
    );
    if (
        showEditingControls !== prevShowEditingControls ||
        editingNote !== prevEditingNote ||
        annotation.note !== prevAnnotationNote
    ) {
        if (showEditingControls !== prevShowEditingControls)
            setPrevShowEditingControls(showEditingControls);
        if (editingNote !== prevEditingNote) setPrevEditingNote(editingNote);
        if (annotation.note !== prevAnnotationNote)
            setPrevAnnotationNote(annotation.note);
        if (!showEditingControls && editingNote) {
            setEditingNote(false);
        } else if (editingNote) {
            setDraftNote(annotation.note ?? '');
        }
    }

    useEffect(() => {
        if (editingNote) {
            requestAnimationFrame(() => {
                if (textareaRef.current) {
                    const ta = textareaRef.current;
                    ta.focus();
                    ta.setSelectionRange(ta.value.length, ta.value.length);
                }
            });
        }
    }, [editingNote]);

    const dotClass = isHighlight ? colorDotClass(annotation.color) : '';
    const quoteBarClass = isHighlight
        ? colorQuoteBarGradient(annotation.color)
        : '';
    const DrawerIcon = isHighlight
        ? (DRAWER_ICONS[annotation.drawer ?? 'lighten'] ?? DRAWER_ICONS.lighten)
        : null;

    const hasWriteCapabilities =
        showEditingControls &&
        (onSaveNote || onColorChange || onDrawerChange || onDelete);
    const showToolbar = hasWriteCapabilities && !editingNote;

    const stopEditingNote = () => {
        setSuppressToolbarTransition(true);
        setEditingNote(false);
        requestAnimationFrame(() => setSuppressToolbarTransition(false));
    };

    const handleNoteSave = () => {
        const trimmed = draftNote.trim();
        onSaveNote?.(trimmed || null);
        stopEditingNote();
    };

    const deleteLabel = isBookmark
        ? translation.get('delete-bookmark')
        : hasNote
          ? translation.get('delete-highlight-and-note')
          : translation.get('delete-highlight');

    return (
        <article
            className={`bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg overflow-hidden shadow-xs${isBookmark ? ' border-l-3 border-l-orange-400 dark:border-l-orange-500' : ''}`}
        >
            <CardHeader
                variant={variant}
                chapter={annotation.chapter}
                pageno={annotation.pageno}
                formattedDate={formattedDate}
                readerHref={readerHref}
            />

            {/* Highlight body: quote + note */}
            {isHighlight && (hasBody || editingNote) && (
                <div className="p-6">
                    {hasText && (
                        <div className="relative">
                            <div
                                className={`absolute top-0 left-0 w-1 h-full bg-linear-to-b ${quoteBarClass} rounded-full`}
                            ></div>
                            <blockquote className="text-gray-900 dark:text-white text-lg leading-relaxed pl-6 font-light whitespace-pre-wrap">
                                {annotation.text}
                            </blockquote>
                        </div>
                    )}

                    {(hasNote || editingNote) && (
                        <NoteEditor
                            variant="highlight"
                            note={annotation.note}
                            editingNote={editingNote}
                            draftNote={draftNote}
                            onDraftChange={setDraftNote}
                            textareaRef={textareaRef}
                            hasText={hasText}
                        />
                    )}
                </div>
            )}

            {/* Bookmark body: note only */}
            {isBookmark && (hasNote || editingNote) && (
                <div className="px-5 pb-4">
                    <NoteEditor
                        variant="bookmark"
                        note={annotation.note}
                        editingNote={editingNote}
                        draftNote={draftNote}
                        onDraftChange={setDraftNote}
                        textareaRef={textareaRef}
                        hasText={false}
                    />
                </div>
            )}

            {editingNote && (
                <NoteEditingFooter
                    px={px}
                    hasNote={hasNote}
                    onDeleteNote={() => {
                        onSaveNote?.(null);
                        stopEditingNote();
                    }}
                    onCancel={stopEditingNote}
                    onSave={handleNoteSave}
                />
            )}

            <AnimatedToolbar
                visible={!!showToolbar}
                editingNote={editingNote}
                suppressTransition={suppressToolbarTransition}
            >
                {onDelete ? (
                    <div className="flex items-center">
                        <DeleteConfirmation
                            label={deleteLabel}
                            confirming={confirmingDelete}
                            onConfirm={() => {
                                onDelete();
                                setConfirmingDelete(false);
                            }}
                            onStartConfirm={() => setConfirmingDelete(true)}
                            onCancelConfirm={() => setConfirmingDelete(false)}
                        />
                    </div>
                ) : (
                    <div />
                )}

                <div className="flex items-center gap-0.5">
                    {isHighlight && onDrawerChange && DrawerIcon && (
                        <>
                            <Button
                                ref={drawerButtonRef}
                                variant="ghost"
                                size="xs"
                                icon={DrawerIcon}
                                label={translation.get(
                                    'highlight-drawer.aria-label',
                                )}
                                onClick={() =>
                                    setActivePicker(
                                        activePicker === 'drawer'
                                            ? null
                                            : 'drawer',
                                    )
                                }
                                className="p-1.5 hover:text-primary-600 dark:hover:text-primary-400"
                            />
                            {activePicker === 'drawer' && (
                                <HighlightDrawerPicker
                                    anchorRef={drawerButtonRef}
                                    currentDrawer={
                                        annotation.drawer ?? 'lighten'
                                    }
                                    onSelect={(drawer) => {
                                        onDrawerChange(drawer);
                                        setActivePicker(null);
                                    }}
                                    onClose={() => setActivePicker(null)}
                                />
                            )}
                        </>
                    )}

                    {isHighlight && onColorChange && (
                        <>
                            <Button
                                ref={colorButtonRef}
                                variant="ghost"
                                size="xs"
                                label={translation.get(
                                    'highlight-color.aria-label',
                                )}
                                onClick={() =>
                                    setActivePicker(
                                        activePicker === 'color'
                                            ? null
                                            : 'color',
                                    )
                                }
                                className="p-1.5"
                            >
                                <span
                                    className={`w-3.5 h-3.5 rounded-full ${dotClass} border border-black/10 dark:border-white/20`}
                                />
                            </Button>
                            {activePicker === 'color' && (
                                <HighlightColorPicker
                                    anchorRef={colorButtonRef}
                                    currentColor={annotation.color ?? 'yellow'}
                                    onSelect={(color) => {
                                        onColorChange(color);
                                        setActivePicker(null);
                                    }}
                                    onClose={() => setActivePicker(null)}
                                />
                            )}
                        </>
                    )}

                    {onSaveNote && (
                        <Button
                            variant="ghost"
                            size="xs"
                            icon={LuPencil}
                            label={
                                hasNote
                                    ? translation.get('edit-note')
                                    : translation.get('add-note')
                            }
                            onClick={() => setEditingNote(!editingNote)}
                            className="p-1.5 hover:text-primary-600 dark:hover:text-primary-400"
                        />
                    )}
                </div>
            </AnimatedToolbar>
        </article>
    );
}
