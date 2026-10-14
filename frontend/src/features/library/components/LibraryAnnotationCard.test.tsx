import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { LibraryAnnotationCard } from './LibraryAnnotationCard';

vi.mock('../../../shared/i18n', () => ({
    translation: { get: (key: string) => key },
}));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
const container = document.createElement('div');
const root = createRoot(container);
const button = (label: string) => {
    const found = Array.from(container.querySelectorAll('button')).find(
        (element) => element.textContent === label,
    );
    expect(found, label).toBeDefined();
    return found!;
};
beforeEach(() => {
    vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => {
        callback(0);
        return 1;
    });
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
});
afterEach(async () => {
    await act(async () => root.render(null));
    vi.unstubAllGlobals();
});

it('keeps an annotation note open on conflict and closes it after a successful save', async () => {
    const onSave = vi
        .fn<() => Promise<void>>()
        .mockRejectedValue(new Error('conflict'));
    await act(async () =>
        root.render(
            <LibraryAnnotationCard
                annotation={{ id: 'a', note: 'draft' }}
                variant="bookmark"
                showEditingControls
                onSaveNote={onSave}
            />,
        ),
    );
    await act(async () => button('edit-note').click());
    expect(container.querySelector('textarea')!.value).toBe('draft');
    await act(async () => button('save').click());
    expect(container.querySelector('textarea')!.value).toBe('draft');
    onSave.mockResolvedValue(undefined);
    await act(async () => button('save').click());
    expect(container.querySelector('textarea')).toBeNull();
    expect(onSave).toHaveBeenCalledTimes(2);
});

it('retains deletion confirmation on conflict and blocks duplicate submissions', async () => {
    let reject!: (error: Error) => void;
    const onDelete = vi.fn(
        () =>
            new Promise<void>((_resolve, fail) => {
                reject = fail;
            }),
    );
    await act(async () =>
        root.render(
            <LibraryAnnotationCard
                annotation={{ id: 'a', note: 'note' }}
                variant="bookmark"
                showEditingControls
                onDelete={onDelete}
            />,
        ),
    );
    await act(async () => button('delete-bookmark').click());
    await act(async () => {
        button('delete').click();
        button('delete').click();
    });
    expect(onDelete).toHaveBeenCalledOnce();
    await act(async () => reject(new Error('conflict')));
    expect(button('delete')).toBeDefined();
    onDelete.mockResolvedValue(undefined);
    await act(async () => button('delete').click());
    expect(button('delete-bookmark')).toBeDefined();
});
