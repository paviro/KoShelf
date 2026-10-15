import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { MemoryRouter, useLocation } from 'react-router';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { afterEach, expect, it, vi } from 'vitest';
import { fetchJson } from '../../../shared/api-fetch';
import { ArchiveRemoval } from './ArchiveRemoval';

vi.mock('../../../shared/api-fetch', () => ({ fetchJson: vi.fn() }));
vi.mock('../../../shared/i18n', () => ({
    translation: {
        get: (key: string, args?: { days: number }) =>
            args ? `${key}:${args.days}` : key,
    },
}));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });

const container = document.createElement('div');
const root = createRoot(container);
afterEach(async () => {
    await act(async () => root.render(null));
    vi.useRealTimers();
    vi.clearAllMocks();
});

function CurrentLocation() {
    return <output>{useLocation().pathname}</output>;
}

it('requires modal confirmation and lets Cancel dismiss without deleting', async () => {
    vi.useFakeTimers();
    vi.mocked(fetchJson).mockResolvedValue(undefined);
    const queries = new QueryClient();
    await act(async () =>
        root.render(
            <QueryClientProvider client={queries}>
                <MemoryRouter initialEntries={['/books/archived-book']}>
                    <ArchiveRemoval
                        item={{
                            id: 'archived-book',
                            content_type: 'book',
                            archive_retention: 30,
                        }}
                        backHref="/books"
                    />
                    <CurrentLocation />
                </MemoryRouter>
            </QueryClientProvider>,
        ),
    );
    const trigger = container.querySelector('button')!;
    const open = async () => {
        await act(async () => trigger.click());
        await act(async () => vi.advanceTimersByTime(50));
    };
    await open();
    expect(fetchJson).not.toHaveBeenCalled();
    // The dialog is portalled out of the toolbar, so it covers the whole page.
    const dialog = document.body.querySelector('[role="dialog"]')!;
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(dialog.textContent).toContain('library-remove-book-body');
    expect(dialog.textContent).toContain('library-remove-days');
    const cancel = Array.from(dialog.querySelectorAll('button')).find(
        (b) => b.textContent === 'cancel',
    )!;
    await act(async () => cancel.click());
    await act(async () => vi.advanceTimersByTime(350));
    expect(document.body.querySelector('[role="dialog"]')).toBeNull();
    expect(fetchJson).not.toHaveBeenCalled();
    await open();
    const confirm = Array.from(
        document.body.querySelectorAll('[role="dialog"] button'),
    ).find((b) => b.textContent === 'library-remove') as HTMLButtonElement;
    await act(async () => confirm.click());
    expect(fetchJson).toHaveBeenCalledExactlyOnceWith(
        '/api/items/archived-book',
        { method: 'DELETE' },
    );
    expect(container.querySelector('output')?.textContent).toBe('/books');
    queries.clear();
});

it.each([
    ['book', 1, 'library-remove-days:1'],
    ['comic', 7, 'library-remove-days:7'],
    ['book', 90, 'library-remove-days:90'],
    ['comic', 0, 'library-remove-immediate'],
    ['book', 'forever', 'library-remove-forever'],
] as const)(
    'uses the %s wording and configured retention %s',
    async (contentType, policy, expected) => {
        vi.useFakeTimers();
        const queries = new QueryClient();
        await act(async () =>
            root.render(
                <QueryClientProvider client={queries}>
                    <MemoryRouter>
                        <ArchiveRemoval
                            item={{
                                id: 'archive',
                                content_type: contentType,
                                archive_retention: policy,
                            }}
                            backHref="/books"
                        />
                    </MemoryRouter>
                </QueryClientProvider>,
            ),
        );
        await act(async () => container.querySelector('button')!.click());
        await act(async () => vi.advanceTimersByTime(50));
        const dialog = document.body.querySelector('[role="dialog"]')!;
        expect(dialog.textContent).toContain(
            `library-remove-${contentType}-body`,
        );
        expect(dialog.textContent).toContain(expected);
        expect(dialog.textContent).not.toContain('NaN');
        expect(fetchJson).not.toHaveBeenCalled();
        queries.clear();
    },
);
