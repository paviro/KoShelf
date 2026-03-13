import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { HashRouter } from 'react-router-dom';

import './styles/app.css';
import { App } from './App';
import { api } from './shared/api';
import type { SiteResponse } from './shared/contracts';
import { translation } from './shared/i18n';
import { RuntimeUpdatesBridge } from './shared/runtime-updates';
import { initThemePreference } from './shared/theme';

const SERVER_MODE_STORAGE_KEY = 'koshelf_server_mode';

if (window.__KOSHELF_SERVER_MODE !== 'external') {
    window.__KOSHELF_SERVER_MODE = 'internal';

    try {
        localStorage.setItem(
            SERVER_MODE_STORAGE_KEY,
            JSON.stringify('internal'),
        );
    } catch {
        // Ignore storage write failures.
    }
}

initThemePreference();

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            staleTime: 30_000,
            refetchOnWindowFocus: false,
            retry: 1,
        },
    },
});

async function bootstrap(): Promise<void> {
    let initialSite: SiteResponse | null = null;
    try {
        initialSite = await api.getSite();
        queryClient.setQueryData(['site'], initialSite);
    } catch {
        initialSite = null;
    }

    await translation.init(initialSite?.language);

    ReactDOM.createRoot(document.getElementById('root')!).render(
        <React.StrictMode>
            <QueryClientProvider client={queryClient}>
                <RuntimeUpdatesBridge />
                <HashRouter
                    future={{
                        v7_startTransition: true,
                        v7_relativeSplatPath: true,
                    }}
                >
                    <App />
                </HashRouter>
            </QueryClientProvider>
        </React.StrictMode>,
    );
}

void bootstrap();
