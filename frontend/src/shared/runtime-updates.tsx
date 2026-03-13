import { useEffect, useRef } from 'react';
import { QueryClient, useQueryClient } from '@tanstack/react-query';

import { api, isServeMode } from './api';

interface DataChangedPayload {
    revision_epoch: string;
    revision: {
        library: number;
        metadata: number;
        stats: number;
        assets: number;
    };
    domains: string[];
    generated_at: string;
}

const STATIC_SITE_POLL_INTERVAL_MS = 10_000;

const QUERY_PREFIXES_TO_INVALIDATE: ReadonlyArray<readonly string[]> = [
    ['site'],
    ['library-list'],
    ['library-detail'],
    ['statistics-index'],
    ['statistics-week'],
    ['statistics-year'],
    ['calendar-months'],
    ['calendar-month'],
    ['recap-index'],
    ['recap-year'],
];

function parseDataChangedPayload(raw: string): DataChangedPayload | null {
    let parsed: unknown;
    try {
        parsed = JSON.parse(raw) as unknown;
    } catch {
        return null;
    }

    if (!parsed || typeof parsed !== 'object') {
        return null;
    }

    const payload = parsed as Record<string, unknown>;
    if (typeof payload.revision_epoch !== 'string') {
        return null;
    }
    if (typeof payload.generated_at !== 'string') {
        return null;
    }

    return payload as unknown as DataChangedPayload;
}

function parseSiteGeneratedAt(payload: unknown): string | null {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const root = payload as Record<string, unknown>;
    const meta = root.meta;

    if (!meta || typeof meta !== 'object') {
        return null;
    }

    const metaRecord = meta as Record<string, unknown>;
    if (typeof metaRecord.generated_at !== 'string') {
        return null;
    }

    return metaRecord.generated_at;
}

function invalidateRuntimeQueries(queryClient: QueryClient): void {
    for (const queryKey of QUERY_PREFIXES_TO_INVALIDATE) {
        void queryClient.invalidateQueries({ queryKey });
    }
}

export function RuntimeUpdatesBridge() {
    const queryClient = useQueryClient();
    const lastRevisionEpochRef = useRef<string | null>(null);
    const lastGeneratedAtRef = useRef<string | null>(null);

    useEffect(() => {
        if (
            !isServeMode() ||
            typeof window === 'undefined' ||
            typeof EventSource === 'undefined'
        ) {
            return;
        }

        const source = new EventSource('/api/events/stream');

        const handleDataChanged = (event: Event) => {
            const message = event as MessageEvent<string>;
            const payload = parseDataChangedPayload(message.data);
            if (!payload) {
                return;
            }

            if (
                lastRevisionEpochRef.current !== null &&
                payload.revision_epoch === lastRevisionEpochRef.current
            ) {
                return;
            }

            lastRevisionEpochRef.current = payload.revision_epoch;
            invalidateRuntimeQueries(queryClient);
        };

        source.addEventListener('data_changed', handleDataChanged);

        return () => {
            source.removeEventListener('data_changed', handleDataChanged);
            source.close();
        };
    }, [queryClient]);

    useEffect(() => {
        if (isServeMode() || typeof window === 'undefined') {
            return;
        }

        let cancelled = false;
        let pollingInFlight = false;

        const pollSiteVersion = async () => {
            if (pollingInFlight || cancelled) {
                return;
            }

            pollingInFlight = true;
            try {
                const response = await fetch('/data/site.json', {
                    method: 'GET',
                    headers: { Accept: 'application/json' },
                    cache: 'no-store',
                });
                if (!response.ok) {
                    return;
                }

                const payload = (await response.json()) as unknown;
                const generatedAt = parseSiteGeneratedAt(payload);
                if (!generatedAt) {
                    return;
                }

                const previousGeneratedAt = lastGeneratedAtRef.current;
                lastGeneratedAtRef.current = generatedAt;

                if (
                    previousGeneratedAt &&
                    previousGeneratedAt !== generatedAt
                ) {
                    api.clearCache();
                    invalidateRuntimeQueries(queryClient);
                }
            } catch {
                // Ignore transient poll failures. Next interval retries.
            } finally {
                pollingInFlight = false;
            }
        };

        void pollSiteVersion();
        const intervalId = window.setInterval(() => {
            void pollSiteVersion();
        }, STATIC_SITE_POLL_INTERVAL_MS);

        return () => {
            cancelled = true;
            window.clearInterval(intervalId);
        };
    }, [queryClient]);

    return null;
}
