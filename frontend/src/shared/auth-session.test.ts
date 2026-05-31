import { QueryClient } from '@tanstack/react-query';
import { afterEach, describe, expect, it } from 'vitest';

import {
    installUnauthorizedSessionCacheSync,
    markSiteAuthenticated,
} from './auth-session';
import type { SiteData } from './contracts';

const siteData: SiteData = {
    title: 'KoShelf',
    language: 'en',
    capabilities: {
        has_books: true,
        has_comics: false,
        has_reading_data: true,
    },
    auth: {
        authenticated: true,
        password_policy: {
            min_chars: 8,
        },
    },
};

describe('auth-session cache helpers', () => {
    const cleanupCallbacks: Array<() => void> = [];

    afterEach(() => {
        while (cleanupCallbacks.length > 0) {
            cleanupCallbacks.pop()?.();
        }
    });

    it('marks cached site data unauthenticated when the unauthorized event fires', () => {
        const queryClient = new QueryClient();
        queryClient.setQueryData(['site'], siteData);
        cleanupCallbacks.push(installUnauthorizedSessionCacheSync(queryClient));

        window.dispatchEvent(new Event('koshelf:unauthorized-session'));

        expect(queryClient.getQueryData<SiteData>(['site'])?.auth).toEqual({
            authenticated: false,
            password_policy: {
                min_chars: 8,
            },
        });
    });

    it('leaves site data without auth unchanged', () => {
        const queryClient = new QueryClient();
        const publicSite: SiteData = {
            ...siteData,
            auth: undefined,
        };
        queryClient.setQueryData(['site'], publicSite);

        markSiteAuthenticated(queryClient, false);

        expect(queryClient.getQueryData<SiteData>(['site'])).toEqual(
            publicSite,
        );
    });

    it('can mark cached site data authenticated after login', () => {
        const queryClient = new QueryClient();
        queryClient.setQueryData(['site'], {
            ...siteData,
            auth: {
                ...siteData.auth!,
                authenticated: false,
            },
        });

        markSiteAuthenticated(queryClient, true);

        expect(
            queryClient.getQueryData<SiteData>(['site'])?.auth?.authenticated,
        ).toBe(true);
    });
});
