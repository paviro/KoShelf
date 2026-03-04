import { Navigate, Route, Routes } from 'react-router-dom';

import { RoutePlaceholder } from './RoutePlaceholder';
import { StatisticsRoute } from '../../features/statistics/routes/StatisticsRoute';
import { CalendarRoute } from '../../features/calendar/routes/CalendarRoute';
import { LibraryDetailRoute } from '../../features/library/routes/LibraryDetailRoute';
import { LibraryListRoute } from '../../features/library/routes/LibraryListRoute';
import { translation } from '../../shared/i18n';

type PlaceholderRoute = {
    path: string;
    titleKey: string;
};

const PLACEHOLDER_ROUTES: PlaceholderRoute[] = [
    { path: '/recap', titleKey: 'recap' },
    { path: '/recap/:year', titleKey: 'recap' },
    { path: '/recap/:year/:scope', titleKey: 'recap' },
];

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
        <Routes>
            <Route
                path="/"
                element={<RootRedirect defaultRoute={defaultRoute} siteLoaded={siteLoaded} />}
            />
            <Route path="/statistics" element={<StatisticsRoute />} />
            <Route path="/statistics/:scope" element={<StatisticsRoute />} />
            <Route path="/calendar" element={<CalendarRoute />} />
            <Route path="/books" element={<LibraryListRoute collection="books" />} />
            <Route path="/books/:id" element={<LibraryDetailRoute collection="books" />} />
            <Route path="/books/:id/:slug" element={<LibraryDetailRoute collection="books" />} />
            <Route path="/comics" element={<LibraryListRoute collection="comics" />} />
            <Route path="/comics/:id" element={<LibraryDetailRoute collection="comics" />} />
            <Route path="/comics/:id/:slug" element={<LibraryDetailRoute collection="comics" />} />

            {PLACEHOLDER_ROUTES.map((route) => (
                <Route
                    key={route.path}
                    path={route.path}
                    element={<RoutePlaceholder title={translation.get(route.titleKey)} />}
                />
            ))}

            <Route path="*" element={<Navigate to={defaultRoute} replace />} />
        </Routes>
    );
}
