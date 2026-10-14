import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, expect, it, vi } from 'vitest';
import { LibraryReviewSection } from './LibraryReviewSection';

vi.mock('../../../shared/i18n', () => ({
    translation: { get: (key: string) => key },
}));
vi.mock('../../../shared/ui/sections/CollapsibleSection', () => ({
    CollapsibleSection: ({
        children,
        controls,
    }: {
        children: React.ReactNode;
        controls: React.ReactNode;
    }) => (
        <div>
            {controls}
            {children}
        </div>
    ),
}));
vi.mock('../components/EditSectionButton', () => ({
    EditSectionButton: ({ onToggle }: { onToggle: () => void }) => (
        <button onClick={onToggle}>edit</button>
    ),
}));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
const container = document.createElement('div');
const root = createRoot(container);
afterEach(async () => {
    await act(async () => root.render(null));
});

it('keeps the review draft and disables save/delete after a background revision change', async () => {
    const onSave = vi.fn();
    const onDelete = vi.fn();
    const render = (revision: string, note: string) =>
        root.render(
            <LibraryReviewSection
                note={note}
                rating={3}
                visible
                onToggle={() => {}}
                canWrite
                metadataRevision={revision}
                onSave={onSave}
                onDelete={onDelete}
            />,
        );
    await act(async () => render('A', 'original note'));
    await act(async () =>
        (container.querySelector('button') as HTMLButtonElement).click(),
    );
    const textarea = container.querySelector('textarea')!;
    expect(textarea.value).toBe('original note');
    await act(async () => render('B', 'external edit'));
    expect(container.querySelector('textarea')!.value).toBe('original note');
    expect(container.querySelector('[role="status"]')?.textContent).toContain(
        'metadata-edit-stale',
    );
    const buttons = Array.from(container.querySelectorAll('button'));
    const save = buttons.find((button) => button.textContent === 'save')!;
    const remove = buttons.find(
        (button) => button.textContent === 'delete-review',
    )!;
    expect(save.disabled).toBe(true);
    expect(remove.disabled).toBe(true);
    await act(async () => {
        save.click();
        remove.click();
    });
    expect(onSave).not.toHaveBeenCalled();
    expect(onDelete).not.toHaveBeenCalled();
});

it('retains a rejected draft, prevents duplicate saves, and closes only on success', async () => {
    let reject!: (error: Error) => void;
    const onSave = vi.fn(
        () =>
            new Promise<void>((_resolve, fail) => {
                reject = fail;
            }),
    );
    await act(async () =>
        root.render(
            <LibraryReviewSection
                note="my draft"
                rating={3}
                visible
                onToggle={() => {}}
                canWrite
                metadataRevision="A"
                onSave={onSave}
            />,
        ),
    );
    await act(async () =>
        (container.querySelector('button') as HTMLButtonElement).click(),
    );
    const save = () =>
        Array.from(container.querySelectorAll('button')).find(
            (b) => b.textContent === 'save',
        )!;
    await act(async () => {
        save().click();
        save().click();
    });
    expect(onSave).toHaveBeenCalledOnce();
    expect(save().disabled).toBe(true);
    await act(async () => reject(new Error('conflict')));
    expect(container.querySelector('textarea')!.value).toBe('my draft');
    expect(save().disabled).toBe(false);
    onSave.mockImplementation(() => Promise.resolve());
    await act(async () => save().click());
    expect(container.querySelector('textarea')).toBeNull();
});
