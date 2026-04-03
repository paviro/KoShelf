export type ButtonVariant = 'outline' | 'neutral' | 'gradient' | 'ghost';
export type ButtonColor =
    | 'primary'
    | 'secondary'
    | 'danger'
    | 'purple'
    | 'blue'
    | 'green';
type ButtonSize = keyof typeof SIZE_CLASSES;

const COLOR_TOKENS: Record<
    ButtonColor,
    { text: string; border: string; hoverBg: string; gradientBg?: string }
> = {
    primary: {
        text: 'text-primary-600 dark:text-primary-400',
        border: 'border-primary-500/30 dark:border-primary-500/20',
        hoverBg: 'not-disabled:hover:bg-primary-50 dark:not-disabled:hover:bg-primary-500/10',
        gradientBg:
            'bg-linear-to-r from-primary-600 to-primary-500 not-disabled:hover:from-primary-500 not-disabled:hover:to-primary-400 shadow-lg shadow-primary-500/20',
    },
    secondary: {
        text: 'text-gray-500 dark:text-dark-400',
        border: 'border-gray-300/50 dark:border-dark-700/50',
        hoverBg: 'not-disabled:hover:bg-gray-100 dark:not-disabled:hover:bg-dark-700',
    },
    danger: {
        text: 'text-red-700 dark:text-red-400',
        border: 'border-red-300/50 dark:border-red-500/30',
        hoverBg: 'not-disabled:hover:bg-red-50 dark:not-disabled:hover:bg-red-500/10',
    },
    purple: {
        text: 'text-purple-600 dark:text-purple-400',
        border: 'border-purple-300/50 dark:border-purple-500/30',
        hoverBg: 'not-disabled:hover:bg-purple-100 dark:not-disabled:hover:bg-purple-900/30',
    },
    blue: {
        text: 'text-blue-600 dark:text-blue-400',
        border: 'border-blue-300/50 dark:border-blue-500/30',
        hoverBg: 'not-disabled:hover:bg-blue-100 dark:not-disabled:hover:bg-blue-900/30',
    },
    green: {
        text: 'text-green-600 dark:text-green-400',
        border: 'border-green-300/50 dark:border-green-500/30',
        hoverBg: 'not-disabled:hover:bg-green-100 dark:not-disabled:hover:bg-green-900/30',
    },
};

const DEFAULT_COLORS: Record<ButtonVariant, ButtonColor> = {
    outline: 'primary',
    neutral: 'secondary',
    gradient: 'primary',
    ghost: 'secondary',
};

const PRIMARY_ACTIVE =
    'bg-primary-50 dark:bg-primary-500/10 border-primary-300/50 dark:border-primary-500/30 text-primary-600 dark:text-primary-400';

function resolveVariantClasses(
    variant: ButtonVariant,
    color: ButtonColor,
    active: boolean,
): string {
    const tokens = COLOR_TOKENS[color];

    switch (variant) {
        case 'outline':
            return active
                ? `border ${PRIMARY_ACTIVE}`
                : `${tokens.text} border ${tokens.border} ${tokens.hoverBg}`;

        case 'neutral':
            return active
                ? `border ${PRIMARY_ACTIVE} backdrop-blur-xs duration-200`
                : 'bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 text-gray-900 dark:text-white not-disabled:hover:bg-gray-200/50 dark:not-disabled:hover:bg-dark-700/50 backdrop-blur-xs duration-200';

        case 'gradient':
            return `font-semibold text-white ${tokens.gradientBg ?? COLOR_TOKENS.primary.gradientBg} transition-all`;

        case 'ghost':
            return active
                ? `bg-primary-50 dark:bg-primary-500/10 text-primary-600 dark:text-primary-400`
                : `${tokens.text} ${tokens.hoverBg}`;
    }
}

const SIZE_CLASSES = {
    sm: 'h-10 px-4 py-2 text-sm gap-2 rounded-lg',
    compact: 'w-10 h-10 p-2.5 rounded-lg',
    xs: 'h-auto px-2 py-1.5 text-sm gap-1.5 rounded-md',
} as const;

const BASE_CLASS =
    'inline-flex items-center justify-center font-medium transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50 disabled:opacity-60';

export type ButtonVariantsOptions = {
    variant?: ButtonVariant | null;
    color?: ButtonColor;
    size?: ButtonSize;
    active?: boolean;
    fullWidth?: boolean;
    className?: string;
};

export function buttonVariants({
    variant = 'outline',
    color,
    size = 'sm',
    active = false,
    fullWidth = false,
    className,
}: ButtonVariantsOptions = {}): string {
    const resolvedColor =
        color ?? (variant ? DEFAULT_COLORS[variant] : 'primary');

    return [
        BASE_CLASS,
        variant && resolveVariantClasses(variant, resolvedColor, active),
        SIZE_CLASSES[size],
        fullWidth && 'w-full',
        className,
    ]
        .filter(Boolean)
        .join(' ');
}
