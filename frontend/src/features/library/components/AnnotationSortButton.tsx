import { LuCalendarArrowDown, LuCalendarArrowUp } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import type { AnnotationSortOrder } from '../lib/annotation-sort';

type AnnotationSortButtonProps = {
    order: AnnotationSortOrder;
    onToggle: () => void;
};

export function AnnotationSortButton({
    order,
    onToggle,
}: AnnotationSortButtonProps) {
    const ariaLabel =
        order === 'desc'
            ? translation.get('sort-order.newest-first')
            : translation.get('sort-order.oldest-first');

    return (
        <Button
            variant="neutral"
            icon={order === 'desc' ? LuCalendarArrowDown : LuCalendarArrowUp}
            aria-label={ariaLabel}
            onClick={onToggle}
        />
    );
}
