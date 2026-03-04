import { useQuery } from '@tanstack/react-query';
import { useLocation } from 'react-router-dom';

import { AppRoutes } from './app/routes/AppRoutes';
import { AppShell } from './app/shell/AppShell';
import { buildNavItems } from './app/shell/shell-nav';
import { api } from './shared/api';
import type { SiteResponse } from './shared/contracts';

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

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const site = siteQuery.data;
    const navItems = buildNavItems(site);
    const defaultRoute = resolveDefaultRoute(site);

    return (
        <AppShell
            navItems={navItems}
            currentPath={location.pathname}
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
