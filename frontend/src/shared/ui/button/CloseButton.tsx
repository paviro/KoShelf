import { LuX } from 'react-icons/lu';

import { translation } from '../../i18n';
import { Button } from './Button';

type CloseButtonProps = {
    onClick: () => void;
    bordered?: boolean;
    disabled?: boolean;
    className?: string;
};

export function CloseButton({
    onClick,
    bordered = false,
    disabled = false,
    className = '',
}: CloseButtonProps) {
    return (
        <Button
            variant={bordered ? 'neutral' : 'ghost'}
            icon={LuX}
            aria-label={translation.get('close.aria-label')}
            onClick={onClick}
            disabled={disabled}
            className={`${bordered ? '' : 'not-disabled:hover:text-gray-900 dark:not-disabled:hover:text-white '}${className}`}
        />
    );
}
