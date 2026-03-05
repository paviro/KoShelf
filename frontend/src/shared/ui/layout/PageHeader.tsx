import { useMemo } from 'react';
import type { ReactNode } from 'react';

import { useRouteHeader } from '../../../app/shell/use-route-header';

type PageHeaderProps = {
    title: string;
    controls?: ReactNode;
};

export function PageHeader({ title, controls }: PageHeaderProps) {
    const header = useMemo(
        () => ({
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
            controls: <div className="flex items-center">{controls}</div>,
        }),
        [controls, title],
    );

    useRouteHeader(header);
    return null;
}
