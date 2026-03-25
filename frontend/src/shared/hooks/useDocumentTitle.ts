import { useEffect } from 'react';

export function useDocumentTitle(
    pageTitle: string | undefined,
    siteTitle: string | undefined,
): void {
    useEffect(() => {
        if (pageTitle && siteTitle) {
            document.title = `${pageTitle} - ${siteTitle}`;
        }
    }, [pageTitle, siteTitle]);
}
