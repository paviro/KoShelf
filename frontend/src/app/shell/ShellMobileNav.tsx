import { Link } from 'react-router';

import { isActivePath, type NavItem } from './shell-nav';

type ShellMobileNavProps = {
    navItems: NavItem[];
    currentPath: string;
};

export function ShellMobileNav({ navItems, currentPath }: ShellMobileNavProps) {
    if (navItems.length <= 1) {
        return null;
    }

    return (
        <nav
            data-shell-nav
            className="lg:hidden fixed bottom-4 left-8 right-8 z-50"
        >
            <div className="bg-white/75 dark:bg-dark-950/75 backdrop-blur-xs border border-gray-200/50 dark:border-dark-700/50 rounded-2xl px-2 py-1.5 shadow-2xl">
                <div className="flex items-center justify-around overflow-x-auto scrollbar-hide">
                    {navItems.map((item) => {
                        const active = isActivePath(currentPath, item.routeId);
                        const ItemIcon = item.icon;

                        return (
                            <Link
                                key={item.href}
                                id={item.id}
                                to={item.href}
                                className={`nav-item flex flex-col items-center py-1.5 px-2 rounded-xl min-w-fit ${active ? 'nav-item-active' : ''}`}
                            >
                                <ItemIcon
                                    className="w-4 h-4 mb-0.5"
                                    aria-hidden="true"
                                />
                                <span className="text-xs">{item.label}</span>
                            </Link>
                        );
                    })}
                </div>
            </div>
        </nav>
    );
}
