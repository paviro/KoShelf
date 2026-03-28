import {
    forwardRef,
    type ButtonHTMLAttributes,
    type ComponentType,
    type ReactNode,
    type SVGAttributes,
} from 'react';

import { buttonVariants, type ButtonVariantsOptions } from './button-variants';

const ICON_SIZE_CLASSES = {
    sm: 'w-4 h-4 shrink-0',
    compact: 'w-5 h-5',
    xs: 'w-3.5 h-3.5 shrink-0',
};

type IconComponent = ComponentType<SVGAttributes<SVGElement>>;

type ButtonProps = Omit<ButtonVariantsOptions, 'size'> &
    Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'className' | 'color'> & {
        icon?: IconComponent;
        iconPosition?: 'start' | 'end';
        label?: string;
        size?: 'sm' | 'xs';
        children?: ReactNode;
    };

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
    function Button(
        {
            variant = 'outline',
            color,
            size = 'sm',
            active,
            fullWidth = false,
            className,
            icon: Icon,
            iconPosition = 'start',
            label,
            children,
            type = 'button',
            ...rest
        },
        ref,
    ) {
        const isIconOnly = !!Icon && !children;
        const effectiveSize = isIconOnly ? 'compact' : size;
        const hasResponsiveLabel = isIconOnly && !!label;

        return (
            <button
                ref={ref}
                type={type}
                className={buttonVariants({
                    variant,
                    color,
                    size: effectiveSize,
                    active,
                    fullWidth,
                    className: hasResponsiveLabel
                        ? `w-auto gap-1.5 px-2.5 sm:px-3.5 ${className ?? ''}`
                        : className,
                })}
                title={
                    isIconOnly || hasResponsiveLabel
                        ? (rest.title ?? label)
                        : rest.title
                }
                aria-label={
                    isIconOnly || hasResponsiveLabel
                        ? (rest['aria-label'] ?? label)
                        : rest['aria-label']
                }
                aria-pressed={active}
                {...rest}
            >
                {Icon && iconPosition === 'start' && (
                    <Icon
                        className={ICON_SIZE_CLASSES[effectiveSize]}
                        aria-hidden="true"
                    />
                )}
                {hasResponsiveLabel && (
                    <span className="hidden sm:inline text-sm">{label}</span>
                )}
                {children}
                {Icon && iconPosition === 'end' && (
                    <Icon
                        className={ICON_SIZE_CLASSES[effectiveSize]}
                        aria-hidden="true"
                    />
                )}
            </button>
        );
    },
);
