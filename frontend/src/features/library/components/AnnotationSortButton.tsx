import { LuArrowDownUp } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import type { AnnotationSortOrder } from '../hooks/useAnnotationSortOrder';

type AnnotationSortButtonProps = {
    order: AnnotationSortOrder;
    onToggle: () => void;
};

export function AnnotationSortButton({
    order,
    onToggle,
}: AnnotationSortButtonProps) {
    const ariaLabel =
        order === 'asc'
            ? translation.get('annotation-sort.aria-label-ascending')
            : translation.get('annotation-sort.aria-label-descending');

    return (
        <Button
            variant="neutral"
            icon={LuArrowDownUp}
            aria-label={ariaLabel}
            onClick={onToggle}
            active={order === 'desc'}
        />
    );
}
