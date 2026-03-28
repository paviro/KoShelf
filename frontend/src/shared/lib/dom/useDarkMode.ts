import { useEffect, useState } from 'react';

export function readDarkMode(): boolean {
    return document.documentElement.classList.contains('dark');
}

export function useDarkMode(): boolean {
    const [dark, setDark] = useState(readDarkMode);
    useEffect(() => {
        const observer = new MutationObserver(() => setDark(readDarkMode()));
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['class'],
        });
        return () => observer.disconnect();
    }, []);
    return dark;
}
