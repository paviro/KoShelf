import type { IconType } from 'react-icons';

type ReaderDrawerEmptyStateVariant = 'contents' | 'highlights' | 'bookmarks';

type ReaderDrawerEmptyStateProps = {
    icon: IconType;
    title: string;
    description: string;
    variant: ReaderDrawerEmptyStateVariant;
};

const VARIANT_CLASSES: Record<
    ReaderDrawerEmptyStateVariant,
    { gradient: string; glow: string }
> = {
    contents: {
        gradient: 'from-blue-500 to-cyan-500',
        glow: 'from-blue-500/25 to-cyan-500/20',
    },
    highlights: {
        gradient: 'from-amber-500 to-orange-500',
        glow: 'from-amber-500/25 to-orange-500/20',
    },
    bookmarks: {
        gradient: 'from-yellow-500 to-amber-500',
        glow: 'from-yellow-500/25 to-amber-500/20',
    },
};

export function ReaderDrawerEmptyState({
    icon: Icon,
    title,
    description,
    variant,
}: ReaderDrawerEmptyStateProps) {
    const classes = VARIANT_CLASSES[variant];

    return (
        <div className="flex h-full min-h-[240px] items-center justify-center px-3 py-8">
            <div className="w-full text-center">
                <div className="relative mx-auto mb-4 h-14 w-14">
                    <div
                        className={`absolute inset-0 rounded-full bg-linear-to-br ${classes.glow} blur-lg`}
                    />
                    <div
                        className={`relative h-14 w-14 rounded-full bg-linear-to-br ${classes.gradient} flex items-center justify-center shadow-md`}
                    >
                        <Icon className="w-6 h-6 text-white" aria-hidden />
                    </div>
                </div>
                <p className="text-[0.95rem] font-semibold text-gray-900 dark:text-white">
                    {title}
                </p>
                <p className="mt-1.5 text-sm font-medium leading-snug text-gray-500 dark:text-dark-300">
                    {description}
                </p>
            </div>
        </div>
    );
}
