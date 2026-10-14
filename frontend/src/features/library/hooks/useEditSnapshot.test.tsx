import { act, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useEditSnapshot } from './useEditSnapshot';

// React's concurrent test environment requires explicit act support.
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });

const container = document.createElement('div');
const root = createRoot(container);
afterEach(async () => {
    await act(async () => root.render(null));
});

describe('edit revision snapshots', () => {
    it('pins data and mutation callbacks until the user closes and reviews again', async () => {
        const saveA = vi.fn();
        const saveB = vi.fn();
        let session: ReturnType<
            typeof useEditSnapshot<{ note: string; save: () => void }>
        >;
        function Editor({
            revision,
            editing,
            note,
            save,
        }: {
            revision: string;
            editing: boolean;
            note: string;
            save: () => void;
        }) {
            const current = useEditSnapshot({ note, save }, revision, editing);
            useEffect(() => {
                session = current;
            });
            return null;
        }
        await act(async () =>
            root.render(
                <Editor revision="A" editing note="old note" save={saveA} />,
            ),
        );
        await act(async () =>
            root.render(
                <Editor revision="B" editing note="new note" save={saveB} />,
            ),
        );
        expect(session!.stale).toBe(true);
        expect(session!.value.note).toBe('old note');
        session!.value.save();
        expect(saveA).toHaveBeenCalledOnce();
        expect(saveB).not.toHaveBeenCalled();
        await act(async () =>
            root.render(
                <Editor
                    revision="B"
                    editing={false}
                    note="new note"
                    save={saveB}
                />,
            ),
        );
        await act(async () =>
            root.render(
                <Editor revision="B" editing note="new note" save={saveB} />,
            ),
        );
        expect(session!.stale).toBe(false);
        expect(session!.value.note).toBe('new note');
        session!.value.save();
        expect(saveB).toHaveBeenCalledOnce();
    });

    it('retains annotation IDs across refresh and deletion confirmation', async () => {
        let value: { id: string }[] = [];
        let stale = false;
        function Editor({
            annotations,
            revision,
        }: {
            annotations: { id: string }[];
            revision: string;
        }) {
            const session = useEditSnapshot(annotations, revision, true);
            useEffect(() => {
                value = session.value;
                stale = session.stale;
            });
            return null;
        }
        await act(async () =>
            root.render(
                <Editor annotations={[{ id: 'original' }]} revision="A" />,
            ),
        );
        await act(async () =>
            root.render(<Editor annotations={[]} revision="B" />),
        );
        expect(value).toEqual([{ id: 'original' }]);
        expect(stale).toBe(true);
    });
});
