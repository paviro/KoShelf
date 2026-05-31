import type { QueryClient } from '@tanstack/react-query';

import type { SiteData } from './contracts';

const UNAUTHORIZED_SESSION_EVENT = 'koshelf:unauthorized-session';
const SITE_QUERY_KEY = ['site'] as const;

export function emitUnauthorizedSessionEvent(): void {
    if (typeof window === 'undefined') {
        return;
    }

    window.dispatchEvent(new Event(UNAUTHORIZED_SESSION_EVENT));
}

export function markSiteAuthenticated(
    queryClient: QueryClient,
    authenticated: boolean,
): void {
    queryClient.setQueryData<SiteData>(SITE_QUERY_KEY, (site) => {
        if (!site?.auth) {
            return site;
        }

        if (site.auth.authenticated === authenticated) {
            return site;
        }

        return {
            ...site,
            auth: {
                ...site.auth,
                authenticated,
            },
        };
    });
}

export function installUnauthorizedSessionCacheSync(
    queryClient: QueryClient,
): () => void {
    if (typeof window === 'undefined') {
        return () => {};
    }

    const handleUnauthorizedSession = () => {
        markSiteAuthenticated(queryClient, false);
    };

    window.addEventListener(
        UNAUTHORIZED_SESSION_EVENT,
        handleUnauthorizedSession,
    );

    return () => {
        window.removeEventListener(
            UNAUTHORIZED_SESSION_EVENT,
            handleUnauthorizedSession,
        );
    };
}
