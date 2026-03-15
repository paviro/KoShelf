import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useLocation } from 'react-router';

import { AppRoutes } from './app/routes/AppRoutes';
import { AppShell } from './app/shell/AppShell';
import { buildNavItems } from './app/shell/shell-nav';
import { api } from './shared/api';
import type { SiteData } from './shared/contracts';
import { I18N_LANGUAGE_CHANGE_EVENT, translation } from './shared/i18n';

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

    return (
        <AppShell
            navItems={navItems}
            currentPath={location.pathname}
            siteTitle={site?.title ?? 'KoShelf'}
            generatedAt={site?.generated_at}
            version={site?.version}
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
