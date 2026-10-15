import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it, vi } from 'vitest';
import { AvailabilityBadge } from '../features/library/components/AvailabilityBadge';
import { documentAvailable } from './library-availability';

vi.mock('./i18n', () => ({
    translation: {
        get: (key: string) =>
            ({
                'library-metadata': 'Metadata',
                'library-archived': 'Archived',
            })[key],
    },
}));

describe('document availability', () => {
    it('gates document actions by availability', () => {
        expect(documentAvailable({ availability: 'metadata' })).toBe(false);
        expect(documentAvailable({ availability: 'archived' })).toBe(false);
        expect(documentAvailable({ availability: 'available' })).toBe(true);
        expect(documentAvailable(undefined)).toBe(false);
    });
    it('renders distinct metadata and archive badges', () => {
        const metadata = renderToStaticMarkup(
            <AvailabilityBadge availability="metadata" />,
        );
        const archive = renderToStaticMarkup(
            <AvailabilityBadge availability="archived" />,
        );
        expect(metadata).toContain('Metadata');
        expect(archive).toContain('Archived');
        expect(metadata).not.toEqual(archive);
    });
});
