import { Navigate, Route, Routes } from 'react-router-dom';

import { PlaceholderPage } from '../pages/PlaceholderPage';
import { StatisticsPage } from '../pages/StatisticsPage';
import { translation } from '../shared/i18n';

type PlaceholderRoute = {
    path: string;
    titleKey: string;
};

const PLACEHOLDER_ROUTES: PlaceholderRoute[] = [
    { path: '/books', titleKey: 'books' },
    { path: '/books/:id', titleKey: 'books' },
    { path: '/books/:id/:slug', titleKey: 'books' },
    { path: '/comics', titleKey: 'comics' },
    { path: '/comics/:id', titleKey: 'comics' },
    { path: '/comics/:id/:slug', titleKey: 'comics' },
    { path: '/calendar', titleKey: 'calendar' },
    { path: '/recap', titleKey: 'recap' },
    { path: '/recap/:year', titleKey: 'recap' },
    { path: '/recap/:year/:scope', titleKey: 'recap' },
];

export function AppRoutes() {
    return (
        <Routes>
            <Route path="/" element={<Navigate to="/statistics" replace />} />
            <Route path="/statistics" element={<StatisticsPage />} />
            <Route path="/statistics/:scope" element={<StatisticsPage />} />

            {PLACEHOLDER_ROUTES.map((route) => (
                <Route
                    key={route.path}
                    path={route.path}
                    element={<PlaceholderPage title={translation.get(route.titleKey)} />}
                />
            ))}

            <Route path="*" element={<Navigate to="/statistics" replace />} />
        </Routes>
    );
}
