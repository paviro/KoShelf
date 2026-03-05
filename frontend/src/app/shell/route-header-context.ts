import { createContext, type ReactNode } from 'react';

export type RouteHeaderConfig = {
    mobileContent: ReactNode;
    desktopContent?: ReactNode;
    controls?: ReactNode;
};

export type RouteHeaderContextValue = {
    setHeader: (header: RouteHeaderConfig) => void;
};

export const RouteHeaderContext = createContext<RouteHeaderContextValue | null>(
    null,
);
