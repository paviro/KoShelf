import { LuX } from 'react-icons/lu';

import { translation } from '../../i18n';
import { Button } from './Button';

type CloseButtonProps = {
    onClick: () => void;
    bordered?: boolean;
    className?: string;
};

export function CloseButton({
    onClick,
    bordered = false,
    className = '',
}: CloseButtonProps) {
    return (
        <Button
            variant={bordered ? 'neutral' : 'ghost'}
            size="icon"
            icon={LuX}
            label={translation.get('close.aria-label')}
            onClick={onClick}
            className={`${bordered ? '' : 'hover:text-gray-900 dark:hover:text-white '}${className}`}
        />
    );
}
