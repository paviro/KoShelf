import { useEffect, useRef, useState } from 'react';
import { LuNotebookPen, LuStar, LuTrash2 } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import { EditSectionButton } from '../components/EditSectionButton';
import { StarRatingInput } from '../components/StarRatingInput';
import { useEditToggle } from '../hooks/useEditToggle';
import { normalizeRating } from '../lib/library-detail-formatters';

type LibraryReviewSectionProps = {
    note: string;
    rating: number | null;
    visible: boolean;
    onToggle: () => void;
    canWrite?: boolean;
    onSave?: (note: string, rating: number) => void;
    onDelete?: () => void;
    saving?: boolean;
    guardedAction?: (action: () => void) => void;
};

export function LibraryReviewSection({
    note,
    rating,
    visible,
    onToggle,
    canWrite = false,
    onSave,
    onDelete,
    saving = false,
    guardedAction,
}: LibraryReviewSectionProps) {
    const normalizedRating = normalizeRating(rating);

    const {
        editing,
        toggle: toggleEditing,
        close: closeEditing,
    } = useEditToggle(guardedAction);
    const [draftNote, setDraftNote] = useState(note);
    const [draftRating, setDraftRating] = useState(normalizedRating);
    const textareaRef = useRef<HTMLTextAreaElement>(null);

    const hasNote = note.trim().length > 0;

    // Sync drafts when entering edit mode or when source values change while editing.
    const [prevEditing, setPrevEditing] = useState(editing);
    const [prevNote, setPrevNote] = useState(note);
    const [prevRating, setPrevRating] = useState(normalizedRating);
    if (
        editing !== prevEditing ||
        note !== prevNote ||
        normalizedRating !== prevRating
    ) {
        if (editing !== prevEditing) setPrevEditing(editing);
        if (note !== prevNote) setPrevNote(note);
        if (normalizedRating !== prevRating) setPrevRating(normalizedRating);
        if (editing) {
            setDraftNote(note);
            setDraftRating(normalizedRating);
        }
    }

    useEffect(() => {
        if (editing && textareaRef.current) {
            const ta = textareaRef.current;
            const len = ta.value.length;
            ta.focus();
            ta.setSelectionRange(len, len);
        }
    }, [editing]);

    const handleSave = () => {
        const trimmed = draftNote.trim();
        onSave?.(trimmed, draftRating);
        closeEditing();
    };

    const handleDelete = () => {
        onDelete?.();
        closeEditing();
    };

    const controls = (
        <div className="flex items-center space-x-4">
            {!editing && normalizedRating > 0 && (
                <div className="flex items-center space-x-1">
                    {Array.from({ length: 5 }, (_, index) => {
                        const filled = index < normalizedRating;
                        return (
                            <LuStar
                                key={index}
                                className={`w-5 h-5 ${
                                    filled
                                        ? 'text-yellow-400 fill-yellow-400'
                                        : 'text-gray-300 dark:text-dark-500'
                                }`}
                                aria-hidden="true"
                            />
                        );
                    })}
                </div>
            )}

            {canWrite && visible && (
                <EditSectionButton editing={editing} onToggle={toggleEditing} />
            )}
        </div>
    );

    return (
        <CollapsibleSection
            sectionKey="review"
            defaultVisible
            accentClass="bg-linear-to-b from-green-400 to-green-600"
            title={translation.get('my-review')}
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
            controls={controls}
        >
            {editing ? (
                <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg overflow-hidden">
                    <div className="p-6 space-y-4">
                        <StarRatingInput
                            value={draftRating}
                            onChange={setDraftRating}
                            size="md"
                        />

                        <textarea
                            ref={textareaRef}
                            value={draftNote}
                            onChange={(e) => setDraftNote(e.target.value)}
                            rows={5}
                            className="w-full rounded-lg border border-gray-200 dark:border-dark-700 bg-gray-50 dark:bg-dark-800 text-gray-900 dark:text-white p-4 text-base leading-relaxed focus:ring-2 focus:ring-primary-500/30 focus:border-primary-500 resize-y placeholder:text-gray-400 dark:placeholder:text-dark-500"
                            placeholder={translation.get('add-review')}
                        />
                    </div>

                    <div className="flex items-center justify-between px-6 py-3 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30">
                        <div>
                            {(note.trim().length > 0 ||
                                normalizedRating > 0) && (
                                <button
                                    type="button"
                                    onClick={handleDelete}
                                    disabled={saving}
                                    className="inline-flex items-center gap-1.5 px-4 py-2 text-sm font-medium text-red-500 dark:text-red-400 border border-red-300/50 dark:border-red-500/30 rounded-lg hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors disabled:opacity-50"
                                >
                                    <LuTrash2
                                        className="w-3.5 h-3.5"
                                        aria-hidden="true"
                                    />
                                    {translation.get('delete-review')}
                                </button>
                            )}
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={closeEditing}
                                disabled={saving}
                                className="px-4 py-2 text-sm font-medium text-gray-500 dark:text-dark-400 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-dark-700 transition-colors disabled:opacity-50"
                            >
                                {translation.get('cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleSave}
                                disabled={saving}
                                className="px-4 py-2 text-sm font-medium text-primary-600 dark:text-primary-400 border border-primary-500/30 dark:border-primary-500/20 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-500/10 transition-colors disabled:opacity-50"
                            >
                                {translation.get('save')}
                            </button>
                        </div>
                    </div>
                </div>
            ) : hasNote ? (
                <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                    <div className="relative">
                        <div className="absolute top-0 left-0 w-1 h-full bg-linear-to-b from-green-400 to-green-600 rounded-full"></div>
                        <div className="pl-6">
                            <p className="text-gray-700 dark:text-dark-300 leading-relaxed text-lg whitespace-pre-wrap">
                                {note}
                            </p>
                        </div>
                    </div>
                </div>
            ) : (
                <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6 flex items-center gap-4">
                    <div className="w-10 h-10 bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600 rounded-lg flex items-center justify-center shrink-0">
                        <LuNotebookPen
                            className="w-5 h-5 text-green-600 dark:text-white"
                            aria-hidden="true"
                        />
                    </div>
                    <div>
                        <p className="text-sm font-medium text-gray-900 dark:text-white">
                            {translation.get('no-review-available')}
                        </p>
                        <p className="text-sm font-medium text-gray-500 dark:text-dark-400 mt-0.5">
                            {canWrite
                                ? translation.get(
                                      'no-review-available.hint-edit',
                                  )
                                : translation.get(
                                      'no-review-available.hint-readonly',
                                  )}
                        </p>
                    </div>
                </div>
            )}
        </CollapsibleSection>
    );
}
