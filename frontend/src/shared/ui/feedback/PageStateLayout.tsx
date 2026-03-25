import type { ReactNode } from 'react';

type PageStateLayoutProps = {
    icon: ReactNode;
    gradientFrom: string;
    gradientTo: string;
    glowFrom: string;
    glowTo: string;
    title: string;
    description: string;
    layout?: 'page' | 'overlay';
    id?: string;
    children?: ReactNode;
};

export function PageStateLayout({
    icon,
    gradientFrom,
    gradientTo,
    glowFrom,
    glowTo,
    title,
    description,
    layout = 'page',
    id,
    children,
}: PageStateLayoutProps) {
    const containerClassName =
        layout === 'overlay'
            ? 'absolute inset-0 z-20 flex items-center justify-center p-6 md:p-8 text-center'
            : 'page-centered-state flex-col text-center';
    const contentClassName =
        layout === 'overlay'
            ? 'w-full max-w-3xl flex flex-col items-center justify-center'
            : 'flex flex-col items-center justify-center';

    return (
        <section className={containerClassName} id={id}>
            <div className={contentClassName}>
                <div className="relative mb-8">
                    <div
                        className={`absolute inset-0 w-32 h-32 bg-linear-to-br ${glowFrom} ${glowTo} rounded-full blur-2xl`}
                    />
                    <div
                        className={`relative w-24 h-24 bg-linear-to-br ${gradientFrom} ${gradientTo} rounded-2xl flex items-center justify-center shadow-2xl`}
                    >
                        {icon}
                    </div>
                </div>
                <h3 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white mb-4">
                    {title}
                </h3>
                <p className="text-lg text-gray-600 dark:text-dark-300 max-w-2xl leading-relaxed whitespace-pre-line">
                    {description}
                </p>
            </div>
            {children}
        </section>
    );
}
