import type { ReactNode } from 'react';

type PageHeaderProps = {
    title: string;
    controls?: ReactNode;
};

export function PageHeader({ title, controls }: PageHeaderProps) {
    return (
        <header className="fixed top-0 left-0 right-0 lg:left-64 bg-white/90 dark:bg-dark-950/75 backdrop-blur-sm border-b border-gray-200/50 dark:border-dark-700/50 px-4 md:px-6 h-[70px] md:h-[80px] z-40">
            <div className="flex items-center justify-between h-full">
                <div className="lg:hidden flex items-center">
                    <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                        {title}
                    </h1>
                </div>

                <h2 className="hidden lg:block text-2xl font-bold text-gray-900 dark:text-white">
                    {title}
                </h2>

                <div className="flex items-center">{controls}</div>
            </div>
        </header>
    );
}
