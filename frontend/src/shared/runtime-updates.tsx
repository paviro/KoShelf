import { useEffect, useRef } from 'react';
import { QueryClient, useQueryClient } from '@tanstack/react-query';

import { isServeMode } from './api';

interface SnapshotUpdatePayload {
    revision: number;
    generated_at: string;
}

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

function parseSnapshotUpdatePayload(raw: string): SnapshotUpdatePayload | null {
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
    if (typeof payload.revision !== 'number' || !Number.isFinite(payload.revision)) {
        return null;
    }
    if (typeof payload.generated_at !== 'string') {
        return null;
    }

    return {
        revision: payload.revision,
        generated_at: payload.generated_at,
    };
}

function invalidateRuntimeQueries(queryClient: QueryClient): void {
    for (const queryKey of QUERY_PREFIXES_TO_INVALIDATE) {
        void queryClient.invalidateQueries({ queryKey });
    }
}

export function RuntimeUpdatesBridge() {
    const queryClient = useQueryClient();
    const lastRevisionRef = useRef(0);

    useEffect(() => {
        if (!isServeMode() || typeof window === 'undefined' || typeof EventSource === 'undefined') {
            return;
        }

        const source = new EventSource('/api/events/stream');

        const handleSnapshotUpdated = (event: Event) => {
            const message = event as MessageEvent<string>;
            const payload = parseSnapshotUpdatePayload(message.data);
            if (!payload) {
                return;
            }

            if (payload.revision <= lastRevisionRef.current) {
                return;
            }

            lastRevisionRef.current = payload.revision;
            invalidateRuntimeQueries(queryClient);
        };

        source.addEventListener('snapshot_updated', handleSnapshotUpdated);

        return () => {
            source.removeEventListener('snapshot_updated', handleSnapshotUpdated);
            source.close();
        };
    }, [queryClient]);

    return null;
}
