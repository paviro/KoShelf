import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { MemoryRouter } from 'react-router';
import { afterEach, expect, it, vi } from 'vitest';
import type { LibraryDetailItem } from '../api/library-data';
import { LibraryOverviewSection } from './LibraryOverviewSection';

vi.mock('../../../shared/i18n', () => ({
    translation: { get: (key: string) => key, getLanguage: () => 'en' },
}));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
const container = document.createElement('div');
const root = createRoot(container);
afterEach(async () => {
    await act(async () => root.render(null));
});

it('tries the uploaded cover after the previous cover failed', async () => {
    const render = (url: string) =>
        root.render(
            <MemoryRouter>
                <LibraryOverviewSection
                    item={
                        {
                            id: 'book',
                            title: 'Book',
                            authors: [],
                            status: 'unknown',
                            cover_url: url,
                            content_type: 'book',
                            format: 'epub',
                            subjects: [],
                            identifiers: [],
                            search_base_path: '/',
                        } as LibraryDetailItem
                    }
                    collection="books"
                    itemStats={null}
                    completions={null}
                    highlightCount={0}
                    noteCount={0}
                    visible
                    onToggle={() => {}}
                />
            </MemoryRouter>,
        );
    await act(async () => render('/assets/covers/book.webp'));
    await act(async () =>
        container.querySelector('img')!.dispatchEvent(new Event('error')),
    );
    expect(container.querySelector('img')).toBeNull();
    await act(async () => render('/assets/covers/book.webp'));
    expect(container.querySelector('img')).toBeNull();
    await act(async () => render('/assets/covers/book.webp?v=uploaded'));
    expect(container.querySelector('img')?.getAttribute('src')).toBe(
        '/assets/covers/book.webp?v=uploaded',
    );
});
