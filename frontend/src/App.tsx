import { useQuery } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';
import { useLocation, useMatch } from 'react-router-dom';

import { AppRoutes } from './app/routes/AppRoutes';
import { AppShell } from './app/shell/AppShell';
import { buildNavItems } from './app/shell/shell-nav';
import { api } from './shared/api';
import type { RecapIndexResponse, SiteResponse } from './shared/contracts';
import {
    clearLibraryListScrollSnapshot,
    type LibraryCollection,
} from './shared/lib/navigation/library-scroll-restoration';

function resolveDefaultRoute(site: SiteResponse | undefined): '/books' | '/comics' | '/statistics' {
    if (site?.capabilities.has_books) {
        return '/books';
    }

    if (site?.capabilities.has_comics) {
        return '/comics';
    }

    return '/statistics';
}

export function App() {
    const location = useLocation();
    const previousDetailCollectionRef = useRef<LibraryCollection | null>(null);
    const onBooksListRoute = useMatch('/books') !== null;
    const onBooksDetailRoute = useMatch('/books/:id') !== null;
    const onComicsListRoute = useMatch('/comics') !== null;
    const onComicsDetailRoute = useMatch('/comics/:id') !== null;

    const currentDetailCollection: LibraryCollection | null = onBooksDetailRoute
        ? 'books'
        : onComicsDetailRoute
          ? 'comics'
          : null;
    const withinBooksCollection = onBooksListRoute || onBooksDetailRoute;
    const withinComicsCollection = onComicsListRoute || onComicsDetailRoute;

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const recapQuery = useQuery({
        queryKey: ['recap-index', 'all'],
        queryFn: () => api.recap.get<RecapIndexResponse>('all'),
        enabled: Boolean(siteQuery.data?.capabilities.has_recap),
    });

    const site = siteQuery.data;
    const navItems = buildNavItems(site, recapQuery.data);
    const defaultRoute = resolveDefaultRoute(site);

    useEffect(() => {
        const previousDetailCollection = previousDetailCollectionRef.current;
        previousDetailCollectionRef.current = currentDetailCollection;

        if (!previousDetailCollection) {
            return;
        }

        const stillWithinPreviousCollection =
            previousDetailCollection === 'books' ? withinBooksCollection : withinComicsCollection;
        if (!stillWithinPreviousCollection) {
            clearLibraryListScrollSnapshot();
        }
    }, [
        currentDetailCollection,
        withinBooksCollection,
        withinComicsCollection,
    ]);

    return (
        <AppShell
            navItems={navItems}
            currentPath={location.pathname}
            currentDetailCollection={currentDetailCollection}
            siteTitle={site?.title ?? 'KoShelf'}
            generatedAt={site?.meta.generated_at}
            version={site?.meta.version}
        >
            <div className="min-h-full">
                <AppRoutes
                    defaultRoute={defaultRoute}
                    siteLoaded={siteQuery.isSuccess || siteQuery.isError}
                />
            </div>
        </AppShell>
    );
}
