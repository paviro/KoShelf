import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { afterEach, expect, it, vi } from 'vitest';
import { fetchJson } from './api-fetch';
import { resolveMediaUrls } from './app-url';
import { StaticApiClient } from './static-api-client';
import { RuntimeUpdatesBridge } from './runtime-updates';
import { api } from './api';

vi.mock('./api', () => ({
    api: { clearCache: vi.fn() },
    isServeMode: () => false,
    isApiHttpError: () => false,
}));

Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
let root: Root | undefined;
afterEach(async () => {
    if (root) await act(async () => root!.unmount());
    root = undefined;
    document.querySelector('base')?.remove();
    vi.restoreAllMocks();
    vi.clearAllMocks();
    vi.useRealTimers();
});

function setBase(href: string) {
    const base = document.createElement('base');
    base.setAttribute('href', href);
    document.head.append(base);
}

it.each(['/', '/koshelf/', '/api/hassio_ingress/token/'])(
    'resolves nested recap and calendar covers and downloads under %s',
    async (base) => {
        setBase(base);
        const item = {
            item_cover: '/assets/covers/book.webp?v=42',
            title: '/assets/unchanged',
        };
        const payload = {
            data: { items: [item], groups: [{ items: [item] }] },
        };
        const fetchMock = vi
            .spyOn(window, 'fetch')
            .mockResolvedValue(new Response(JSON.stringify(payload)));
        const resolvedItem = {
            ...item,
            item_cover: `${base}assets/covers/book.webp?v=42`,
        };
        await expect(
            fetchJson('/api/reading/completions?year=2026'),
        ).resolves.toEqual({
            data: {
                items: [resolvedItem],
                groups: [{ items: [resolvedItem] }],
            },
        });
        expect(fetchMock).toHaveBeenCalledWith(
            `${base}api/reading/completions?year=2026`,
            expect.any(Object),
        );
        expect(resolveMediaUrls({ days: [{ items: [item] }] })).toEqual({
            days: [{ items: [resolvedItem] }],
        });
        expect(new StaticApiClient().getItemDownloadHref('book')).toBe(
            `${base}data/items/book.json`,
        );
    },
);

it('preserves missing, external, and already-prefixed covers and ordinary text', () => {
    setBase('/koshelf/');
    const payload = {
        items: [
            {},
            { item_cover: null },
            { item_cover: '' },
            { item_cover: 'https://example.com/assets/cover.webp' },
            { item_cover: '//example.com/assets/cover.webp' },
            { item_cover: '/koshelf/assets/covers/book.webp?v=42' },
        ],
        title: '/assets/title',
        review_note: '/assets/review',
    };
    expect(resolveMediaUrls(payload)).toEqual(payload);
});

it.each(['/', '/koshelf/', '/api/hassio_ingress/token/'])(
    'polls beneath %s and invalidates cached data only when the generation changes',
    async (base) => {
        setBase(base);
        vi.useFakeTimers();
        let generation = '2026-10-15T10:00:00Z';
        const fetchMock = vi
            .spyOn(window, 'fetch')
            .mockImplementation(
                async () =>
                    new Response(
                        JSON.stringify({ meta: { generated_at: generation } }),
                    ),
            );
        const client = new QueryClient();
        client.setQueryData(['recap-year', 2026], { items: [] });
        client.setQueryData(['calendar-month', '2026-10'], { days: [] });
        root = createRoot(document.createElement('div'));
        await act(async () =>
            root!.render(
                <QueryClientProvider client={client}>
                    <RuntimeUpdatesBridge />
                </QueryClientProvider>,
            ),
        );
        expect(fetchMock).toHaveBeenCalledWith(`${base}data/site.json`, {
            method: 'GET',
            headers: { Accept: 'application/json' },
            cache: 'no-store',
        });
        await act(async () => vi.advanceTimersByTimeAsync(10_000));
        expect(api.clearCache).not.toHaveBeenCalled();
        expect(client.getQueryState(['recap-year', 2026])?.isInvalidated).toBe(
            false,
        );
        generation = '2026-10-15T11:00:00Z';
        await act(async () => vi.advanceTimersByTimeAsync(10_000));
        expect(api.clearCache).toHaveBeenCalledTimes(1);
        expect(client.getQueryState(['recap-year', 2026])?.isInvalidated).toBe(
            true,
        );
        expect(
            client.getQueryState(['calendar-month', '2026-10'])?.isInvalidated,
        ).toBe(true);
    },
);
