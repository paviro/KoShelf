import { useCallback, useRef, useState } from 'react';

const STORAGE_KEY = 'koshelf_edit_warning_dismissed';

function isWarningDismissed(): boolean {
    try {
        return localStorage.getItem(STORAGE_KEY) === 'true';
    } catch {
        return false;
    }
}

function dismissWarning(): void {
    try {
        localStorage.setItem(STORAGE_KEY, 'true');
    } catch {
        // Ignore storage errors.
    }
}

/**
 * One-time write warning gate. Call `guardedAction(callback)` — if the
 * warning has been permanently dismissed the callback runs immediately,
 * otherwise the modal is shown and the callback runs after acknowledging.
 */
export function useEditWarning() {
    const [warningOpen, setWarningOpen] = useState(false);
    const [dismissed, setDismissed] = useState(isWarningDismissed);
    const pendingRef = useRef<(() => void) | null>(null);

    const guardedAction = useCallback(
        (action: () => void) => {
            if (dismissed) {
                action();
                return;
            }
            pendingRef.current = action;
            setWarningOpen(true);
        },
        [dismissed],
    );

    const acknowledge = useCallback((dontShowAgain: boolean) => {
        if (dontShowAgain) {
            dismissWarning();
            setDismissed(true);
        }
        setWarningOpen(false);
        const action = pendingRef.current;
        pendingRef.current = null;
        action?.();
    }, []);

    const cancel = useCallback(() => {
        setWarningOpen(false);
        pendingRef.current = null;
    }, []);

    return { guardedAction, warningOpen, acknowledge, cancel };
}
