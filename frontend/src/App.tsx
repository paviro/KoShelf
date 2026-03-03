import { useQuery } from '@tanstack/react-query';
import { useLocation } from 'react-router-dom';

import { AppRoutes } from './app/AppRoutes';
import { AppShell } from './features/layout/AppShell';
import { buildNavItems } from './features/layout/shell-nav';
import { api } from './shared/api';
import type { RecapIndexResponse, SiteResponse } from './shared/contracts';

export function App() {
    const location = useLocation();

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

    return (
        <AppShell
            navItems={navItems}
            currentPath={location.pathname}
            siteTitle={site?.title ?? 'KoShelf'}
            generatedAt={site?.meta.generated_at}
            version={site?.meta.version}
        >
            <div className="min-h-full">
                <AppRoutes />
            </div>
        </AppShell>
    );
}
