import { useMemo, useState, type ReactNode } from 'react';

import { translation } from '../../shared/i18n';
import { matchRoute } from '../routes/route-registry';
import {
    RouteHeaderContext,
    type RouteHeaderConfig,
    type RouteHeaderContextValue,
} from './route-header-context';

function resolveFallbackTitle(pathname: string, siteTitle: string): string {
    const routeId = matchRoute(pathname).routeId;
    switch (routeId) {
        case 'statistics':
            return translation.get('reading-statistics');
        case 'calendar':
            return translation.get('calendar');
        case 'settings':
            return translation.get('settings');
        case 'books-list':
        case 'books-detail':
        case 'books-read':
            return translation.get('books');
        case 'comics-list':
        case 'comics-detail':
        case 'comics-read':
            return translation.get('comics');
        case 'recap':
            return translation.get('recap');
        default:
            return siteTitle;
    }
}

function createFallbackHeader(
    pathname: string,
    siteTitle: string,
): RouteHeaderConfig {
    const title = resolveFallbackTitle(pathname, siteTitle);
    return {
        mobileContent: (
            <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                {title}
            </h1>
        ),
        desktopContent: (
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white truncate">
                {title}
            </h2>
        ),
    };
}

type RouteHeaderProviderProps = {
    currentPath: string;
    siteTitle: string;
    children: ReactNode;
};

export function RouteHeaderProvider({
    currentPath,
    siteTitle,
    children,
}: RouteHeaderProviderProps) {
    const [headerForPath, setHeaderForPath] = useState<{
        path: string;
        config: RouteHeaderConfig;
    } | null>(null);

    const fallbackHeader = useMemo(
        () => createFallbackHeader(currentPath, siteTitle),
        [currentPath, siteTitle],
    );
    const activeHeader =
        headerForPath?.path === currentPath
            ? headerForPath.config
            : fallbackHeader;
    const desktopContent =
        activeHeader.desktopContent ?? activeHeader.mobileContent;
    const contextValue = useMemo<RouteHeaderContextValue>(
        () => ({
            setHeader: (header) => {
                setHeaderForPath({
                    path: currentPath,
                    config: header,
                });
            },
        }),
        [currentPath],
    );

    return (
        <RouteHeaderContext.Provider value={contextValue}>
            <header className="fixed top-0 left-0 right-0 lg:left-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-xs border-b border-gray-200/50 dark:border-dark-700/50 px-4 md:px-6 h-[70px] md:h-[80px] z-40">
                <div className="flex items-center justify-between h-full">
                    <div className="lg:hidden flex items-center min-w-0 flex-1">
                        {activeHeader.mobileContent}
                    </div>
                    <div className="hidden lg:flex items-center min-w-0 flex-1">
                        {desktopContent}
                    </div>
                    {activeHeader.controls}
                </div>
            </header>
            {children}
        </RouteHeaderContext.Provider>
    );
}
