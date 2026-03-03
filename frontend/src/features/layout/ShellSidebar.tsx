import { Link } from 'react-router-dom';

import { translation } from '../../shared/i18n';
import { BRAND_ICON_SVG, isActivePath, type NavItem } from './shell-nav';

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
    return (
        <aside className="hidden lg:flex fixed left-0 top-0 bottom-0 w-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-sm border-r border-gray-200/50 dark:border-dark-700/50 flex-col z-30">
            <div className="py-4 px-6 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 bg-gradient-to-br from-primary-400 to-primary-600 rounded-xl flex items-center justify-center shadow-lg">
                        <svg
                            className="w-6 h-6 text-white"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d={BRAND_ICON_SVG}
                            ></path>
                        </svg>
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
                {navItems.map((item) => {
                    const active = isActivePath(currentPath, item.href);
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
                                <svg
                                    className={`w-4 h-4 transition-colors duration-200 ${
                                        active
                                            ? 'text-white'
                                            : 'text-gray-700 dark:text-dark-200 group-hover:text-primary-600 dark:group-hover:text-primary-300'
                                    }`}
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth="2"
                                        d={item.iconSvg}
                                    ></path>
                                </svg>
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

            <div className="mt-auto px-4 py-4 border-t border-gray-200/50 dark:border-dark-700/50">
                <div className="bg-gray-100/50 dark:bg-dark-900/50 border border-gray-200/50 dark:border-dark-700/50 rounded-lg p-3">
                    <div className="flex items-center mb-2">
                        <div className="w-6 h-6 bg-gradient-to-br from-gray-500 to-gray-600 rounded-lg flex items-center justify-center mr-2">
                            <svg
                                className="w-3 h-3 text-white"
                                fill="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"></path>
                            </svg>
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
                            &middot; v{version ?? '--'}
                        </span>
                    </div>
                    <div className="text-xs text-gray-400 dark:text-dark-500">
                        {translation.get('last-updated')}: {generatedAt ?? '--'}
                    </div>
                </div>
            </div>
        </aside>
    );
}
