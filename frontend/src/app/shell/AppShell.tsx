import type { ReactNode } from 'react';

import type { LibraryCollection } from '../../shared/lib/navigation/library-scroll-restoration';
import { ShellMobileNav } from './ShellMobileNav';
import { ShellSidebar } from './ShellSidebar';
import type { NavItem } from './shell-nav';

type AppShellProps = {
    navItems: NavItem[];
    currentPath: string;
    currentDetailCollection: LibraryCollection | null;
    siteTitle: string;
    generatedAt?: string;
    version?: string;
    children: ReactNode;
};

export function AppShell({
    navItems,
    currentPath,
    currentDetailCollection,
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
                currentDetailCollection={currentDetailCollection}
                siteTitle={siteTitle}
                generatedAt={generatedAt}
                version={version}
            />
            <ShellMobileNav
                navItems={navItems}
                currentPath={currentPath}
                currentDetailCollection={currentDetailCollection}
            />

            <div className="min-h-full lg:ml-64">{children}</div>
        </div>
    );
}
