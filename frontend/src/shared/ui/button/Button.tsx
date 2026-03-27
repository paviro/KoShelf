import {
    forwardRef,
    type ButtonHTMLAttributes,
    type ComponentType,
    type ReactNode,
    type SVGAttributes,
} from 'react';

import { buttonVariants, type ButtonVariantsOptions } from './button-variants';

const ICON_SIZE_CLASSES: Record<string, string> = {
    sm: 'w-4 h-4 shrink-0',
    icon: 'w-5 h-5',
    xs: 'w-3.5 h-3.5 shrink-0',
};

type IconComponent = ComponentType<SVGAttributes<SVGElement>>;

type ButtonProps = ButtonVariantsOptions &
    Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'className' | 'color'> & {
        icon?: IconComponent;
        label?: string;
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
            label,
            children,
            type = 'button',
            ...rest
        },
        ref,
    ) {
        const isIconOnly = size === 'icon' && !children;

        return (
            <button
                ref={ref}
                type={type}
                className={buttonVariants({
                    variant,
                    color,
                    size,
                    active,
                    fullWidth,
                    className,
                })}
                title={isIconOnly ? (rest.title ?? label) : rest.title}
                aria-label={
                    isIconOnly
                        ? (rest['aria-label'] ?? label)
                        : rest['aria-label']
                }
                aria-pressed={active}
                {...rest}
            >
                {Icon && (
                    <Icon
                        className={ICON_SIZE_CLASSES[size]}
                        aria-hidden="true"
                    />
                )}
                {children}
            </button>
        );
    },
);
