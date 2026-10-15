import { afterEach, expect, it } from 'vitest';
import { appUrl, resolveMediaUrls } from './app-url';
afterEach(() => document.querySelector('base')?.remove());
it('keeps root hosting unchanged', () =>
    expect(appUrl('/api/site')).toBe('/api/site'));
it('resolves APIs and media under ingress without altering book text', () => {
    const base = document.createElement('base');
    base.href = '/api/hassio_ingress/token/';
    document.head.append(base);
    expect(appUrl('/api/events/stream')).toBe(
        '/api/hassio_ingress/token/api/events/stream',
    );
    expect(appUrl('/#/login')).toBe('/api/hassio_ingress/token/#/login');
    expect(
        resolveMediaUrls({
            cover_url: '/assets/covers/a.webp',
            title: '/assets/a',
        }),
    ).toEqual({
        cover_url: '/api/hassio_ingress/token/assets/covers/a.webp',
        title: '/assets/a',
    });
});
