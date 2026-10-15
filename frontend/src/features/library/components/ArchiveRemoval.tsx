import { useState } from 'react';
import { createPortal } from 'react-dom';
import { LuTrash2 } from 'react-icons/lu';
import { useNavigate } from 'react-router';
import { useQueryClient } from '@tanstack/react-query';
import { fetchJson } from '../../../shared/api-fetch';
import type { LibraryDetailItem } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';

export function ArchiveRemoval({
    item,
    backHref,
}: {
    item: Pick<LibraryDetailItem, 'id' | 'content_type' | 'archive_retention'>;
    backHref: string;
}) {
    const [open, setOpen] = useState(false);
    const [pending, setPending] = useState(false);
    const [error, setError] = useState(false);
    const navigate = useNavigate();
    const queries = useQueryClient();
    const policy = item.archive_retention;
    const explanation =
        policy === 'forever'
            ? translation.get('library-remove-forever')
            : policy === 0
              ? translation.get('library-remove-immediate')
              : translation.get('library-remove-days', { days: policy ?? 0 });
    async function remove() {
        setPending(true);
        setError(false);
        try {
            await fetchJson(`/api/items/${encodeURIComponent(item.id)}`, {
                method: 'DELETE',
            });
            await queries.invalidateQueries();
            navigate(backHref);
        } catch {
            setError(true);
        } finally {
            setPending(false);
        }
    }
    return (
        <>
            <Button
                variant="neutral"
                icon={LuTrash2}
                title={translation.get('library-remove')}
                aria-label={translation.get('library-remove')}
                aria-haspopup="dialog"
                onClick={() => {
                    setError(false);
                    setOpen(true);
                }}
            />
            {createPortal(
                <ModalShell
                    open={open}
                    onClose={() => {
                        if (!pending) setOpen(false);
                    }}
                    containerClassName="z-[60]"
                    cardClassName="max-w-md bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-6"
                >
                    <h2 className="mb-4 pr-10 text-lg font-semibold text-gray-900 dark:text-white">
                        {translation.get('library-remove')}
                    </h2>
                    <p>
                        {translation.get(
                            item.content_type === 'comic'
                                ? 'library-remove-comic-body'
                                : 'library-remove-book-body',
                        )}
                    </p>
                    <p className="mt-2">{explanation}</p>
                    {error && (
                        <p role="alert" className="mt-3 text-red-600">
                            {translation.get('library-remove-error')}
                        </p>
                    )}
                    <div className="mt-6 flex justify-end gap-2">
                        <Button
                            color="secondary"
                            disabled={pending}
                            onClick={() => setOpen(false)}
                        >
                            {translation.get('cancel')}
                        </Button>
                        <Button
                            color="danger"
                            disabled={pending}
                            onClick={() => void remove()}
                        >
                            {translation.get('library-remove')}
                        </Button>
                    </div>
                </ModalShell>,
                document.body,
            )}
        </>
    );
}
