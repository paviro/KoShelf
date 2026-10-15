import { useCallback, useState } from 'react';

const STORAGE_KEY = 'koshelf_koreader_cache_notice_dismissed';

function isNoticeDismissed(): boolean {
    try {
        return localStorage.getItem(STORAGE_KEY) === 'true';
    } catch {
        return false;
    }
}

function dismissNotice(): void {
    try {
        localStorage.setItem(STORAGE_KEY, 'true');
    } catch {
        // Ignore storage errors.
    }
}

/**
 * Post-save notice that KOReader may show cached book details until refreshed.
 * `show()` does nothing once the notice has been permanently dismissed.
 */
export function useCacheNotice() {
    const [noticeOpen, setNoticeOpen] = useState(false);

    const show = useCallback(() => {
        if (!isNoticeDismissed()) setNoticeOpen(true);
    }, []);

    const close = useCallback((dontShowAgain: boolean) => {
        if (dontShowAgain) dismissNotice();
        setNoticeOpen(false);
    }, []);

    return { noticeOpen, show, close };
}
