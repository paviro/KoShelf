import { useRef, useState } from 'react';

/** Keep a draft open until its mutation succeeds, and reject duplicate clicks. */
export function usePendingEdit() {
    const active = useRef(false);
    const [pending, setPending] = useState(false);
    const run = async (
        action: () => void | Promise<void>,
        onSuccess: () => void,
    ) => {
        if (active.current) return;
        active.current = true;
        setPending(true);
        try {
            await action();
            onSuccess();
        } catch {
            // Mutation hooks report errors and refresh the source. Keep the draft
            // and its pinned revision so a failed save never silently rebases it.
        } finally {
            active.current = false;
            setPending(false);
        }
    };
    return { pending, run };
}
