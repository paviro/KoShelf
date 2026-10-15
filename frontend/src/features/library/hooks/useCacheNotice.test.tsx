import { act, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, expect, it } from 'vitest';
import { useCacheNotice } from './useCacheNotice';

Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
afterEach(() => localStorage.clear());

it('shows after each save until it is dismissed for good', async () => {
    const root = createRoot(document.createElement('div'));
    let notice: ReturnType<typeof useCacheNotice>;
    function Editor() {
        const current = useCacheNotice();
        useEffect(() => {
            notice = current;
        });
        return null;
    }
    await act(async () => root.render(<Editor />));
    await act(async () => notice!.show());
    expect(notice!.noticeOpen).toBe(true);
    await act(async () => notice!.close(false));
    await act(async () => notice!.show());
    expect(notice!.noticeOpen).toBe(true);
    await act(async () => notice!.close(true));
    await act(async () => notice!.show());
    expect(notice!.noticeOpen).toBe(false);
    await act(async () => root.unmount());
});
