import { Navigate, Route, Routes } from 'react-router-dom';

import { StatisticsRoute } from '../../features/statistics/routes/StatisticsRoute';
import { CalendarRoute } from '../../features/calendar/routes/CalendarRoute';
import { LibraryDetailRoute } from '../../features/library/routes/LibraryDetailRoute';
import { LibraryListRoute } from '../../features/library/routes/LibraryListRoute';
import { RecapRoute } from '../../features/recap/routes/RecapRoute';

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
            <Route path="/comics" element={<LibraryListRoute collection="comics" />} />
            <Route path="/comics/:id" element={<LibraryDetailRoute collection="comics" />} />
            <Route path="/recap" element={<RecapRoute />} />
            <Route path="/recap/:year" element={<RecapRoute />} />
            <Route path="/recap/:year/:scope" element={<RecapRoute />} />

            <Route path="*" element={<Navigate to={defaultRoute} replace />} />
        </Routes>
    );
}
