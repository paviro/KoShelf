import type { SVGAttributes } from 'react';

export function SortNewestIcon(props: SVGAttributes<SVGElement>) {
    return (
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" {...props}>
            <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 7h8M4 12h8M4 17h5M18 6v12M15 15l3 3 3-3"
            />
        </svg>
    );
}

export function SortOldestIcon(props: SVGAttributes<SVGElement>) {
    return (
        <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" {...props}>
            <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M4 7h5M4 12h8M4 17h8M18 18V6M15 9l3-3 3 3"
            />
        </svg>
    );
}
