import { useContext, useLayoutEffect } from 'react';

import {
    RouteHeaderContext,
    type RouteHeaderConfig,
} from './route-header-context';

export function useRouteHeader(header: RouteHeaderConfig): void {
    const context = useContext(RouteHeaderContext);

    useLayoutEffect(() => {
        if (!context) {
            return;
        }
        context.setHeader(header);
    }, [context, header]);
}
