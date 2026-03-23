import { useCallback, useState } from 'react';

export function useEditToggle(guardedAction?: (action: () => void) => void) {
    const [editing, setEditing] = useState(false);

    const toggle = useCallback(() => {
        if (editing) {
            setEditing(false);
        } else if (guardedAction) {
            guardedAction(() => setEditing(true));
        } else {
            setEditing(true);
        }
    }, [editing, guardedAction]);

    const close = useCallback(() => setEditing(false), []);

    return { editing, toggle, close };
}
