import { Suspense } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';

import { RouteScrollRestoration } from '../../shared/lib/navigation/RouteScrollRestoration';
import { LoadingSpinner } from '../../shared/ui/feedback/LoadingSpinner';
import { PageContent } from '../../shared/ui/layout/PageContent';
import {
    CalendarRoute,
    LibraryDetailRoute,
    LibraryListRoute,
    RecapRoute,
    SettingsRoute,
    StatisticsRoute,
} from './lazy-routes';
import { routePathPattern } from './route-registry';

type AppRoutesProps = {
    defaultRoute: '/books' | '/comics' | '/statistics';
    siteLoaded: boolean;
};

function RootRedirect({
    defaultRoute,
    siteLoaded,
}: {
    defaultRoute: AppRoutesProps['defaultRoute'];
    siteLoaded: boolean;
}) {
    if (!siteLoaded) {
        return null;
    }

    return <Navigate to={defaultRoute} replace />;
}

function RouteChunkFallback() {
    return (
        <PageContent>
            <section className="min-h-[calc(100vh-14rem)] flex items-center justify-center">
                <LoadingSpinner size="lg" srLabel="Loading page" delayMs={10} />
            </section>
        </PageContent>
    );
}

export function AppRoutes({ defaultRoute, siteLoaded }: AppRoutesProps) {
    return (
        <>
            <RouteScrollRestoration />
            <Suspense fallback={<RouteChunkFallback />}>
                <Routes>
                    <Route
                        path={routePathPattern('root')}
                        element={
                            <RootRedirect
                                defaultRoute={defaultRoute}
                                siteLoaded={siteLoaded}
                            />
                        }
                    />
                    <Route
                        path={routePathPattern('statistics')}
                        element={<StatisticsRoute />}
                    />
                    <Route
                        path={routePathPattern('calendar')}
                        element={<CalendarRoute />}
                    />
                    <Route
                        path={routePathPattern('settings')}
                        element={<SettingsRoute />}
                    />
                    <Route
                        path={routePathPattern('books-list')}
                        element={<LibraryListRoute collection="books" />}
                    />
                    <Route
                        path={routePathPattern('books-detail')}
                        element={<LibraryDetailRoute collection="books" />}
                    />
                    <Route
                        path={routePathPattern('comics-list')}
                        element={<LibraryListRoute collection="comics" />}
                    />
                    <Route
                        path={routePathPattern('comics-detail')}
                        element={<LibraryDetailRoute collection="comics" />}
                    />
                    <Route
                        path={routePathPattern('recap')}
                        element={<RecapRoute />}
                    />

                    <Route
                        path="*"
                        element={<Navigate to={defaultRoute} replace />}
                    />
                </Routes>
            </Suspense>
        </>
    );
}
