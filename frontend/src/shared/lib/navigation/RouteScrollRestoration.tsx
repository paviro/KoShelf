import { useCallback, useEffect, useLayoutEffect, useRef } from 'react';
import { useLocation } from 'react-router';
import {
    isScrollableRouteId,
    matchRoute,
    type ScrollableRouteId,
} from '../../../app/routes/route-registry';
import { patchRouteState, readRouteState } from '../state/route-state-storage';
import { beginScrollRestore, endScrollRestore } from './scroll-restore-state';

const RESTORE_RETRY_INTERVAL_MS = 50;
const MAX_RESTORE_DURATION_MS = 1500;
const MAX_RESTORE_ATTEMPTS = Math.ceil(
    MAX_RESTORE_DURATION_MS / RESTORE_RETRY_INTERVAL_MS,
);
const PERSIST_DEBOUNCE_MS = 150;

function resolveScrollableRouteId(pathname: string): ScrollableRouteId | null {
    const matched = matchRoute(pathname);
    return isScrollableRouteId(matched.routeId) ? matched.routeId : null;
}

function normalizeScrollSnapshot(value: unknown): number {
    if (typeof value !== 'number' || !Number.isFinite(value)) {
        return 0;
    }

    return Math.max(0, Math.floor(value));
}

function readScrollSnapshot(routeId: ScrollableRouteId): number {
    const persisted = readRouteState(routeId, 'session');
    return normalizeScrollSnapshot(persisted.scrollSnapshot);
}

function persistScrollSnapshot(
    routeId: ScrollableRouteId,
    scrollY: number,
): void {
    patchRouteState(routeId, 'session', {
        scrollSnapshot: normalizeScrollSnapshot(scrollY),
    });
}

function resolvePersistScrollY(lastKnownScrollY: number): number {
    const liveScrollY = window.scrollY;

    // Browsers can report 0 during reload teardown even when the user
    // was scrolled down. Fall back to the last observed position in that case.
    if (liveScrollY <= 0 && lastKnownScrollY > 0) {
        return lastKnownScrollY;
    }

    return liveScrollY;
}

function restoreScrollPosition(targetY: number): () => void {
    const html = document.documentElement;
    html.style.overflow = 'hidden';
    beginScrollRestore();

    let attempts = 0;
    const maxAttempts = targetY > 0 ? MAX_RESTORE_ATTEMPTS : 1;
    let timerId: number | null = null;
    let endRestoreFrameId: number | null = null;
    let cancelled = false;
    let hasFinished = false;

    const finish = () => {
        if (hasFinished) {
            return;
        }

        hasFinished = true;
        html.style.overflow = '';
        // Defer ending restore tracking until the next frame so programmatic
        // scroll events triggered by scrollTo are still treated as restore events.
        if (endRestoreFrameId === null) {
            endRestoreFrameId = window.requestAnimationFrame(() => {
                endRestoreFrameId = null;
                endScrollRestore();
            });
        }
    };

    const restore = () => {
        if (cancelled) {
            return;
        }

        window.scrollTo({ top: targetY, behavior: 'auto' });
        attempts += 1;

        const maxScrollableY = Math.max(
            0,
            document.documentElement.scrollHeight - window.innerHeight,
        );
        const reachedTarget = Math.abs(window.scrollY - targetY) <= 1;
        const reachedPageBottom =
            Math.abs(window.scrollY - maxScrollableY) <= 1;
        const pageCanReachTarget = maxScrollableY >= targetY - 1;
        const canStopAtBottom = targetY <= 0 || pageCanReachTarget;
        if (
            reachedTarget ||
            (reachedPageBottom && canStopAtBottom) ||
            attempts >= maxAttempts
        ) {
            finish();
            return;
        }

        timerId = window.setTimeout(restore, RESTORE_RETRY_INTERVAL_MS);
    };

    restore();

    return () => {
        cancelled = true;
        if (timerId !== null) {
            window.clearTimeout(timerId);
        }
        if (endRestoreFrameId !== null) {
            window.cancelAnimationFrame(endRestoreFrameId);
            endRestoreFrameId = null;
            endScrollRestore();
        }
        finish();
    };
}

/**
 * Persists route scroll positions in session storage and restores them when
 * revisiting routes during the same browser tab session.
 */
export function RouteScrollRestoration(): null {
    const { pathname } = useLocation();
    const currentRouteId = resolveScrollableRouteId(pathname);
    const activeRouteIdRef = useRef<ScrollableRouteId | null>(currentRouteId);
    const activeRouteScrollYRef = useRef(window.scrollY);
    const hasHydratedInitialRouteRef = useRef(false);
    const persistTimeoutIdRef = useRef<number | null>(null);
    const pendingPersistRouteIdRef = useRef<ScrollableRouteId | null>(null);
    const pendingPersistScrollYRef = useRef<number>(0);

    const captureScrollSnapshot = useCallback(
        (routeId: ScrollableRouteId | null, scrollY: number) => {
            if (!routeId) {
                return;
            }
            persistScrollSnapshot(routeId, scrollY);
        },
        [],
    );

    const flushPersist = useCallback(() => {
        if (persistTimeoutIdRef.current !== null) {
            window.clearTimeout(persistTimeoutIdRef.current);
            persistTimeoutIdRef.current = null;
        }

        if (pendingPersistRouteIdRef.current !== null) {
            persistScrollSnapshot(
                pendingPersistRouteIdRef.current,
                pendingPersistScrollYRef.current,
            );
            pendingPersistRouteIdRef.current = null;
        }
    }, []);

    const schedulePersist = useCallback(
        (routeId: ScrollableRouteId | null, scrollY: number) => {
            if (!routeId) {
                return;
            }

            pendingPersistRouteIdRef.current = routeId;
            pendingPersistScrollYRef.current = scrollY;
            if (persistTimeoutIdRef.current !== null) {
                window.clearTimeout(persistTimeoutIdRef.current);
            }

            persistTimeoutIdRef.current = window.setTimeout(() => {
                persistTimeoutIdRef.current = null;
                if (pendingPersistRouteIdRef.current === null) {
                    return;
                }

                persistScrollSnapshot(
                    pendingPersistRouteIdRef.current,
                    pendingPersistScrollYRef.current,
                );
                pendingPersistRouteIdRef.current = null;
            }, PERSIST_DEBOUNCE_MS);
        },
        [],
    );

    useLayoutEffect(() => {
        const previousRouteId = activeRouteIdRef.current;
        if (hasHydratedInitialRouteRef.current) {
            captureScrollSnapshot(
                previousRouteId,
                activeRouteScrollYRef.current,
            );
        }
        activeRouteIdRef.current = currentRouteId;

        const targetY = currentRouteId ? readScrollSnapshot(currentRouteId) : 0;
        activeRouteScrollYRef.current = targetY;
        const cleanupRestore = restoreScrollPosition(targetY);
        hasHydratedInitialRouteRef.current = true;
        schedulePersist(currentRouteId, targetY);

        return () => {
            cleanupRestore();
        };
    }, [captureScrollSnapshot, currentRouteId, schedulePersist]);

    useEffect(() => {
        const handleScroll = () => {
            const routeId = activeRouteIdRef.current;
            if (!routeId) {
                return;
            }

            activeRouteScrollYRef.current = window.scrollY;
            schedulePersist(routeId, window.scrollY);
        };

        window.addEventListener('scroll', handleScroll, { passive: true });

        return () => {
            window.removeEventListener('scroll', handleScroll);
            const finalScrollY = resolvePersistScrollY(
                activeRouteScrollYRef.current,
            );
            activeRouteScrollYRef.current = finalScrollY;
            captureScrollSnapshot(activeRouteIdRef.current, finalScrollY);
            flushPersist();
        };
    }, [captureScrollSnapshot, flushPersist, schedulePersist]);

    useEffect(() => {
        const flush = () => {
            const finalScrollY = resolvePersistScrollY(
                activeRouteScrollYRef.current,
            );
            activeRouteScrollYRef.current = finalScrollY;
            captureScrollSnapshot(activeRouteIdRef.current, finalScrollY);
            flushPersist();
        };

        const handleVisibilityChange = () => {
            if (document.visibilityState === 'hidden') {
                flush();
            }
        };

        window.addEventListener('pagehide', flush);
        document.addEventListener('visibilitychange', handleVisibilityChange);

        return () => {
            window.removeEventListener('pagehide', flush);
            document.removeEventListener(
                'visibilitychange',
                handleVisibilityChange,
            );
            flush();
        };
    }, [captureScrollSnapshot, flushPersist]);

    return null;
}
