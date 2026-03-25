import {
    forwardRef,
    type ButtonHTMLAttributes,
    type ComponentType,
    type ReactNode,
    type SVGAttributes,
} from 'react';

import {
    buttonVariants,
    ICON_SIZE_CLASSES,
    type ButtonVariantsOptions,
} from './button-variants';

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
