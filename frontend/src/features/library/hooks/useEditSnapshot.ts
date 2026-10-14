import { useState } from 'react';

/** Keep displayed values and their mutation callbacks on the same revision. */
export function useEditSnapshot<T>(
    value: T,
    revision: string | null,
    editing: boolean,
) {
    const [snapshot, setSnapshot] = useState<{
        value: T;
        revision: string | null;
    } | null>(null);
    if (editing && snapshot === null) {
        setSnapshot({ value, revision });
    } else if (!editing && snapshot !== null) {
        setSnapshot(null);
    }
    const current =
        editing && snapshot !== null ? snapshot : { value, revision };
    return {
        value: current.value,
        stale: editing && current.revision !== revision,
    };
}
