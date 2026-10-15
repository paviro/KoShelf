import { act, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { afterEach, expect, it, vi } from 'vitest';
import { useCustomEdit } from './library-mutations';

const calls: string[] = [];
const api = vi.hoisted(() => ({
    uploadCover: vi.fn(),
    resetCover: vi.fn(),
    updateMetadata: vi.fn(),
    getItem: vi.fn(),
}));
vi.mock('../../../shared/api', () => ({ api }));
vi.mock('../../../shared/i18n', () => ({
    translation: { get: (key: string) => key, getLanguage: () => 'en' },
}));
vi.mock('../../../shared/ui/toast', () => ({ addToast: vi.fn() }));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });

async function mountMutation() {
    const root = createRoot(document.createElement('div'));
    let mutation: ReturnType<typeof useCustomEdit>;
    function Harness() {
        const current = useCustomEdit('book', 'books');
        useEffect(() => {
            mutation = current;
        });
        return null;
    }
    const client = new QueryClient({
        defaultOptions: { mutations: { retry: false } },
    });
    await act(async () =>
        root.render(
            <QueryClientProvider client={client}>
                <Harness />
            </QueryClientProvider>,
        ),
    );
    return { mutate: () => mutation!, unmount: () => root.unmount() };
}

afterEach(() => {
    calls.length = 0;
    vi.clearAllMocks();
});

it('saves the cover first, then the metadata, each with its own revision', async () => {
    api.resetCover.mockImplementation(async (_id, revision) => {
        calls.push(`cover:${revision}`);
    });
    api.updateMetadata.mockImplementation(async (_id, _patch, revision) => {
        calls.push(`metadata:${revision}`);
    });
    const { mutate, unmount } = await mountMutation();
    await act(async () => {
        await mutate().mutateAsync({
            cover: { file: null, revision: 'c1' },
            metadata: { patch: { title: 'New' }, revision: 'm1' },
        });
    });
    expect(calls).toEqual(['cover:c1', 'metadata:m1']);
    expect(api.getItem).not.toHaveBeenCalled();
    unmount();
});

it('does not write the metadata when the cover is rejected', async () => {
    api.uploadCover.mockRejectedValue(new Error('unsupported cover format'));
    const { mutate, unmount } = await mountMutation();
    await act(async () => {
        await mutate()
            .mutateAsync({
                cover: { file: new File(['x'], 'x.png'), revision: 'c1' },
                metadata: { patch: { title: 'New' }, revision: 'm1' },
            })
            .catch(() => {});
    });
    expect(api.updateMetadata).not.toHaveBeenCalled();
    unmount();
});
