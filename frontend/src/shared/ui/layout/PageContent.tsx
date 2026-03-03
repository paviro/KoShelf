import type { ReactNode } from 'react';

type PageContentProps = {
    children: ReactNode;
    className?: string;
};

const PAGE_CONTENT_BASE_CLASSNAME = 'pt-[88px] md:pt-24 pb-28 lg:pb-6 px-4 md:px-6';

export function PageContent({ children, className = '' }: PageContentProps) {
    const resolvedClassName = className
        ? `${PAGE_CONTENT_BASE_CLASSNAME} ${className}`
        : PAGE_CONTENT_BASE_CLASSNAME;

    return <main className={resolvedClassName}>{children}</main>;
}
