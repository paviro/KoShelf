import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { expect, it, vi } from 'vitest';
import { LibraryStatusFilter } from './LibraryStatusFilter';

vi.mock('../../../shared/i18n', () => ({
    translation: { get: (key: string) => key },
}));
Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });

it.each([true, false])(
    'shows availability controls only with archive mode (%s)',
    async (archiveEnabled) => {
        const container = document.createElement('div');
        document.body.append(container);
        const root = createRoot(container);
        try {
            await act(async () =>
                root.render(
                    <LibraryStatusFilter
                        archiveEnabled={archiveEnabled}
                        value="all"
                        options={['all', 'reading']}
                        onChange={vi.fn()}
                        availability="all"
                        onAvailabilityChange={vi.fn()}
                    />,
                ),
            );
            const trigger = container.querySelector('button')!;
            vi.spyOn(trigger, 'getBoundingClientRect').mockReturnValue(
                new DOMRect(100, 100, 100, 40),
            );
            expect(
                trigger
                    .getAttribute('aria-label')!
                    .includes('library-availability'),
            ).toBe(archiveEnabled);
            await act(async () => trigger.click());
            expect(
                document.body.querySelector(
                    '[aria-label="library-availability"]',
                ) !== null,
            ).toBe(archiveEnabled);
            expect(
                document.body.querySelector(
                    '[aria-label="library-reading-status"]',
                ),
            ).not.toBeNull();
        } finally {
            await act(async () => root.unmount());
            container.remove();
        }
    },
);
