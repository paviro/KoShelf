import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { ApiHttpError, fetchJson } from './api-fetch';

const unauthorizedEnvelope = {
    error: {
        code: 'unauthorized',
        message: 'Missing, invalid, or expired session',
    },
};

function jsonResponse(status: number, payload: unknown): Response {
    return new Response(JSON.stringify(payload), {
        status,
        headers: { 'content-type': 'application/json' },
    });
}

describe('fetchJson', () => {
    beforeEach(() => {
        window.location.hash = '#/login';
    });

    afterEach(() => {
        vi.restoreAllMocks();
        window.location.hash = '';
    });

    it('rejects and emits an unauthorized session event for redirect-enabled 401 responses', async () => {
        const fetchMock = vi
            .spyOn(window, 'fetch')
            .mockResolvedValue(jsonResponse(401, unauthorizedEnvelope));
        const unauthorizedListener = vi.fn();
        window.addEventListener(
            'koshelf:unauthorized-session',
            unauthorizedListener,
        );

        await expect(fetchJson('/api/items')).rejects.toMatchObject({
            name: 'ApiHttpError',
            status: 401,
            code: 'unauthorized',
            apiMessage: 'Missing, invalid, or expired session',
        } satisfies Partial<ApiHttpError>);

        expect(fetchMock).toHaveBeenCalledWith('/api/items', {
            method: 'GET',
            headers: { Accept: 'application/json' },
            body: undefined,
            cache: undefined,
        });
        expect(unauthorizedListener).toHaveBeenCalledTimes(1);

        window.removeEventListener(
            'koshelf:unauthorized-session',
            unauthorizedListener,
        );
    });

    it('does not emit an unauthorized session event when redirect handling is disabled', async () => {
        vi.spyOn(window, 'fetch').mockResolvedValue(
            jsonResponse(401, unauthorizedEnvelope),
        );
        const unauthorizedListener = vi.fn();
        window.addEventListener(
            'koshelf:unauthorized-session',
            unauthorizedListener,
        );

        await expect(
            fetchJson('/api/auth/login', {
                method: 'POST',
                redirectOnUnauthorized: false,
            }),
        ).rejects.toMatchObject({
            name: 'ApiHttpError',
            status: 401,
            code: 'unauthorized',
        } satisfies Partial<ApiHttpError>);

        expect(unauthorizedListener).not.toHaveBeenCalled();

        window.removeEventListener(
            'koshelf:unauthorized-session',
            unauthorizedListener,
        );
    });
});
