import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useLocation } from 'react-router';

import { AppRoutes } from './app/routes/AppRoutes';
import { isReaderRouteId, matchRoute } from './app/routes/route-registry';
import { AppShell } from './app/shell/AppShell';
import { buildNavItems } from './app/shell/shell-nav';
import { api } from './shared/api';
import type { SiteData } from './shared/contracts';
import { I18N_LANGUAGE_CHANGE_EVENT, translation } from './shared/i18n';
import { RouteErrorBoundary } from './shared/ui/feedback/RouteErrorBoundary';
import { ToastContainer } from './shared/ui/toast';

function resolveDefaultRoute(
    site: SiteData | undefined,
): '/books' | '/comics' | '/statistics' {
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
    const [, setI18nVersion] = useState(0);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.getSite(),
    });

    const site = siteQuery.data;
    useEffect(() => {
        const language = site?.language;
        if (!language) return;

        void translation.init(language);
    }, [site?.language]);
    useEffect(() => {
        const handleLanguageChange = () => {
            setI18nVersion((value) => value + 1);
        };

        window.addEventListener(
            I18N_LANGUAGE_CHANGE_EVENT,
            handleLanguageChange,
        );
        return () => {
            window.removeEventListener(
                I18N_LANGUAGE_CHANGE_EVENT,
                handleLanguageChange,
            );
        };
    }, []);

    const navItems = buildNavItems(site);
    const defaultRoute = resolveDefaultRoute(site);
    const siteTitle = site?.title ?? 'KoShelf';
    const authenticated = site?.authenticated;
    const routeMatch = matchRoute(location.pathname);
    const isLoginRoute = routeMatch.routeId === 'login';
    const isReaderRoute = isReaderRouteId(routeMatch.routeId);

    const routes = (
        <RouteErrorBoundary>
            <div className="min-h-full">
                <AppRoutes
                    defaultRoute={defaultRoute}
                    siteTitle={siteTitle}
                    authenticated={authenticated}
                    siteLoaded={siteQuery.isSuccess || siteQuery.isError}
                />
            </div>
        </RouteErrorBoundary>
    );

    if (isLoginRoute) {
        return (
            <div className="min-h-full bg-gray-100 dark:bg-dark-925 text-gray-900 dark:text-white font-sans">
                {routes}
                <ToastContainer />
            </div>
        );
    }

    if (isReaderRoute) {
        return (
            <div className="min-h-full">
                {routes}
                <ToastContainer />
            </div>
        );
    }

    return (
        <AppShell
            navItems={navItems}
            currentPath={location.pathname}
            siteTitle={siteTitle}
            generatedAt={site?.generated_at}
            version={site?.version}
        >
            {routes}
            <ToastContainer />
        </AppShell>
    );
}
