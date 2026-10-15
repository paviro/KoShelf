import { afterEach, describe, expect, it, vi } from 'vitest';
import { HttpApiClient } from './http-api-client';
import { addToast } from './ui/toast';

vi.mock('./ui/toast', () => ({ addToast: vi.fn() }));

describe('metadata write revisions', () => {
    afterEach(() => vi.restoreAllMocks());

    it('sends the displayed revision for item, annotation, and deletion requests', async () => {
        const fetchMock = vi
            .spyOn(window, 'fetch')
            .mockResolvedValue(new Response(null, { status: 204 }));
        const client = new HttpApiClient();
        await client.updateItem('book', { rating: 4 }, 'revision-a');
        await client.updateAnnotation(
            'book',
            'note',
            { note: 'edited' },
            'revision-b',
        );
        await client.deleteAnnotation('book', 'note', 'revision-c');
        for (const [index, revision] of [
            'revision-a',
            'revision-b',
            'revision-c',
        ].entries()) {
            expect(fetchMock.mock.calls[index]?.[1]?.headers).toMatchObject({
                'X-Metadata-Revision': revision,
            });
        }
    });

    it('returns a conflict without automatically retrying the mutation', async () => {
        const fetchMock = vi.spyOn(window, 'fetch').mockResolvedValue(
            new Response(
                JSON.stringify({
                    error: { code: 'conflict', message: 'stale revision' },
                }),
                {
                    status: 409,
                    headers: { 'content-type': 'application/json' },
                },
            ),
        );
        await expect(
            new HttpApiClient().deleteAnnotation('book', 'note', 'old'),
        ).rejects.toMatchObject({ status: 409 });
        expect(fetchMock).toHaveBeenCalledTimes(1);
    });
});

it('treats a pending backup as successful and warns without resending the source edit', async () => {
    const fetchMock = vi.spyOn(window, 'fetch').mockResolvedValue(
        new Response(JSON.stringify({ backup_pending: true }), {
            status: 200,
            headers: { 'content-type': 'application/json' },
        }),
    );
    await expect(
        new HttpApiClient().updateItem('book', { rating: 5 }, 'revision'),
    ).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(addToast).toHaveBeenCalledWith('warning', expect.any(String));
    vi.restoreAllMocks();
});

it('sends the custom edit revision with unchanged raw image bytes', async () => {
    const fetchMock = vi
        .spyOn(window, 'fetch')
        .mockResolvedValue(new Response(null, { status: 204 }));
    const api = new HttpApiClient();
    const file = new File(['png'], 'cover.png', { type: 'image/png' });
    await api.updateMetadata(
        'book',
        { title: null, authors: 'One\nTwo' },
        'metadata-rev',
    );
    await api.uploadCover('book', file, 'cover-rev');
    await api.resetCover('book', 'reset-rev');
    expect(fetchMock.mock.calls[0]?.[1]).toMatchObject({
        method: 'PATCH',
        headers: { 'X-Edit-Revision': 'metadata-rev' },
        body: JSON.stringify({ title: null, authors: 'One\nTwo' }),
    });
    expect(fetchMock.mock.calls[1]?.[1]).toMatchObject({
        method: 'PUT',
        headers: { 'X-Edit-Revision': 'cover-rev' },
        body: file,
    });
    expect(fetchMock.mock.calls[2]?.[1]).toMatchObject({
        method: 'DELETE',
        headers: { 'X-Edit-Revision': 'reset-rev' },
    });
    vi.restoreAllMocks();
});
