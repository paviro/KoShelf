import { useEffect, useRef, useState } from 'react';
import { LuNotebookPen, LuStar, LuTrash2 } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
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

    useEffect(() => {
        if (!editing) {
            return;
        }

        const frameId = window.requestAnimationFrame(() => {
            setDraftNote(note);
            setDraftRating(normalizedRating);
        });

        return () => {
            window.cancelAnimationFrame(frameId);
        };
    }, [editing, note, normalizedRating]);

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

    const handleToggleEditing = () => {
        if (!editing) {
            setDraftNote(note);
            setDraftRating(normalizedRating);
        }

        toggleEditing();
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
                <EditSectionButton
                    editing={editing}
                    onToggle={handleToggleEditing}
                />
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
                                <Button
                                    color="danger"
                                    icon={LuTrash2}
                                    onClick={handleDelete}
                                    disabled={saving}
                                >
                                    {translation.get('delete-review')}
                                </Button>
                            )}
                        </div>
                        <div className="flex items-center gap-2">
                            <Button
                                color="secondary"
                                onClick={closeEditing}
                                disabled={saving}
                            >
                                {translation.get('cancel')}
                            </Button>
                            <Button onClick={handleSave} disabled={saving}>
                                {translation.get('save')}
                            </Button>
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
