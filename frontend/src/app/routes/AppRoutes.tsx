import { Suspense, lazy } from 'react';
import { Navigate, Route, Routes } from 'react-router-dom';

import { CalendarRoute } from '../../features/calendar/routes/CalendarRoute';
import { LibraryDetailRoute } from '../../features/library/routes/LibraryDetailRoute';
import { LibraryListRoute } from '../../features/library/routes/LibraryListRoute';
import { RecapRoute } from '../../features/recap/routes/RecapRoute';
import { StatisticsRoute } from '../../features/statistics/routes/StatisticsRoute';
import { RouteScrollRestoration } from '../../shared/lib/navigation/RouteScrollRestoration';
import { routePathPattern } from './route-registry';

const SettingsRoute = lazy(async () => {
    const module = await import('../../features/settings/routes/SettingsRoute');
    return { default: module.SettingsRoute };
});

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

export function AppRoutes({ defaultRoute, siteLoaded }: AppRoutesProps) {
    return (
        <>
            <RouteScrollRestoration />
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
                    element={
                        <Suspense fallback={null}>
                            <SettingsRoute />
                        </Suspense>
                    }
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
        </>
    );
}
