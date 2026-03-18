import { LuChevronDown } from 'react-icons/lu';

const selectClassName =
    'w-full appearance-none bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg pl-3 pr-10 py-2.5 text-gray-900 dark:text-white focus:outline-hidden focus:ring-2 focus:ring-primary-500/60';

type SettingsSelectProps = {
    children: React.ReactNode;
    className?: string;
} & Omit<
    React.SelectHTMLAttributes<HTMLSelectElement>,
    'children' | 'className'
>;

export function SettingsSelect({
    children,
    className,
    ...props
}: SettingsSelectProps) {
    const resolvedClassName = className
        ? `${selectClassName} ${className}`
        : selectClassName;

    return (
        <div className="relative">
            <select {...props} className={resolvedClassName}>
                {children}
            </select>
            <LuChevronDown
                className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-dark-400"
                aria-hidden="true"
            />
        </div>
    );
}
