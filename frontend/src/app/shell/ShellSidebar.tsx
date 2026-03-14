import { Link } from 'react-router';
import { LuGithub, LuSettings } from 'react-icons/lu';
import { translation } from '../../shared/i18n';
import { formatInstant } from '../../shared/lib/intl/formatDate';
import { BRAND_ICON, isActivePath, type NavItem } from './shell-nav';

type ShellSidebarProps = {
    navItems: NavItem[];
    currentPath: string;
    siteTitle: string;
    generatedAt?: string;
    version?: string;
};

export function ShellSidebar({
    navItems,
    currentPath,
    siteTitle,
    generatedAt,
    version,
}: ShellSidebarProps) {
    const BrandIcon = BRAND_ICON;
    const formattedGeneratedAt = formatInstant(generatedAt, {
        dateStyle: 'medium',
        timeStyle: 'short',
    });
    const primaryNavItems = navItems.filter(
        (item) => item.routeId !== 'settings',
    );
    const settingsActive = isActivePath(currentPath, 'settings');

    return (
        <aside className="hidden lg:flex fixed left-0 top-0 bottom-0 w-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-sm border-r border-gray-200/50 dark:border-dark-700/50 flex-col z-30">
            <div className="py-4 px-6 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 bg-gradient-to-br from-primary-400 to-primary-600 rounded-xl flex items-center justify-center shadow-lg">
                        <BrandIcon
                            className="w-6 h-6 text-white"
                            aria-hidden="true"
                        />
                    </div>
                    <div>
                        <h1 className="text-xl font-bold text-gray-900 dark:text-white">
                            {siteTitle}
                        </h1>
                        <p className="text-xs text-gray-500 dark:text-dark-400 mt-0.5">
                            {translation.get('reading-companion')}
                        </p>
                    </div>
                </div>
            </div>

            <nav className="flex-1 px-4 py-6 space-y-3">
                {primaryNavItems.map((item) => {
                    const active = isActivePath(currentPath, item.routeId);
                    const ItemIcon = item.icon;

                    return (
                        <Link
                            key={item.href}
                            id={item.id}
                            to={item.href}
                            className={`sidebar-item-modern group ${active ? 'sidebar-item-modern-active' : ''}`}
                        >
                            <div
                                className={`w-8 h-8 rounded-lg flex items-center justify-center transition-all duration-200 ease-out border-2 border-transparent ${
                                    active
                                        ? 'bg-gradient-to-br from-primary-500 to-primary-600 shadow-lg !border-primary-400/50'
                                        : 'bg-gray-100 dark:bg-dark-700 group-hover:bg-primary-100 dark:group-hover:bg-primary-900/40 group-hover:border-primary-200 dark:group-hover:border-primary-600/50 group-hover:-translate-y-0.5'
                                }`}
                            >
                                <ItemIcon
                                    className={`w-4 h-4 transition-colors duration-200 ${
                                        active
                                            ? 'text-white'
                                            : 'text-gray-700 dark:text-dark-200 group-hover:text-primary-600 dark:group-hover:text-primary-300'
                                    }`}
                                    aria-hidden="true"
                                />
                            </div>
                            <span
                                className={`transition-colors duration-200 ${
                                    active
                                        ? 'text-gray-900 dark:text-white font-semibold'
                                        : 'text-gray-700 dark:text-dark-200 group-hover:text-primary-600 dark:group-hover:text-primary-300 font-medium'
                                }`}
                            >
                                {item.label}
                            </span>
                        </Link>
                    );
                })}
            </nav>

            <div className="mt-auto px-4 py-4 border-t border-gray-200/50 dark:border-dark-700/50 space-y-3">
                <Link
                    to="/settings"
                    className={`block rounded-lg p-3 transition-all duration-200 border ${
                        settingsActive
                            ? 'bg-primary-50/50 dark:bg-primary-900/20 border-primary-200/50 dark:border-primary-700/50'
                            : 'bg-gray-100/50 dark:bg-dark-900/50 border-gray-200/50 dark:border-dark-700/50 hover:bg-gray-200/50 dark:hover:bg-dark-800/50'
                    }`}
                    aria-label={translation.get('settings')}
                >
                    <div className="flex items-center">
                        <div
                            className={`w-6 h-6 rounded-lg flex items-center justify-center mr-2 ${
                                settingsActive
                                    ? 'bg-gradient-to-br from-primary-400 to-primary-500'
                                    : 'bg-gradient-to-br from-gray-500 to-gray-600'
                            }`}
                        >
                            <LuSettings
                                className="w-3 h-3 text-white"
                                aria-hidden="true"
                            />
                        </div>
                        <span
                            className={`text-xs font-medium ${
                                settingsActive
                                    ? 'text-primary-700 dark:text-primary-300'
                                    : 'text-gray-500 dark:text-dark-400'
                            }`}
                        >
                            {translation.get('settings')}
                        </span>
                    </div>
                </Link>
                <div className="bg-gray-100/50 dark:bg-dark-900/50 border border-gray-200/50 dark:border-dark-700/50 rounded-lg p-3">
                    <div className="flex items-center mb-2">
                        <div className="w-6 h-6 bg-gradient-to-br from-gray-500 to-gray-600 rounded-lg flex items-center justify-center mr-2">
                            <LuGithub
                                className="w-3 h-3 text-white"
                                aria-hidden="true"
                            />
                        </div>
                        <a
                            href="https://github.com/paviro/KOShelf"
                            target="_blank"
                            rel="noreferrer"
                            className="text-xs text-gray-500 dark:text-dark-400 hover:text-primary-400 transition-colors font-medium"
                        >
                            {translation.get('github')}
                        </a>
                        <span className="text-xs text-gray-400 dark:text-dark-500 ml-1">
                            &middot; {version ?? '--'}
                        </span>
                    </div>
                    <div className="text-[0.65rem] leading-tight text-gray-400 dark:text-dark-500">
                        {translation.get('last-updated')}:{' '}
                        {formattedGeneratedAt}
                    </div>
                </div>
            </div>
        </aside>
    );
}
