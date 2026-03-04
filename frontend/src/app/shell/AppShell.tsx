import type { ReactNode } from 'react';

import { ShellMobileNav } from './ShellMobileNav';
import { ShellSidebar } from './ShellSidebar';
import type { NavItem } from './shell-nav';

type AppShellProps = {
    navItems: NavItem[];
    currentPath: string;
    siteTitle: string;
    generatedAt?: string;
    version?: string;
    children: ReactNode;
};

export function AppShell({
    navItems,
    currentPath,
    siteTitle,
    generatedAt,
    version,
    children,
}: AppShellProps) {
    return (
        <div className="min-h-full bg-gray-100 dark:bg-dark-925 text-gray-900 dark:text-white font-sans">
            <ShellSidebar
                navItems={navItems}
                currentPath={currentPath}
                siteTitle={siteTitle}
                generatedAt={generatedAt}
                version={version}
            />
            <ShellMobileNav
                navItems={navItems}
                currentPath={currentPath}
            />

            <div className="min-h-full lg:ml-64">{children}</div>
        </div>
    );
}
