import { LuCalendarArrowDown, LuCalendarArrowUp } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import {
    SortNewestIcon as SortDownIcon,
    SortOldestIcon as SortUpIcon,
} from '../../../shared/ui/icons/SortOrderIcons';
import type { AnnotationSortOrder } from '../lib/annotation-sort';

type AnnotationSortButtonProps = {
    order: AnnotationSortOrder;
    onToggle: () => void;
};

const SORT_ORDER_LABELS: Record<AnnotationSortOrder, string> = {
    'date-asc': 'sort-order.oldest-first',
    'date-desc': 'sort-order.newest-first',
    'page-asc': 'sort-order.page-ascending',
    'page-desc': 'sort-order.page-descending',
};

const SORT_ORDER_ICONS = {
    'date-asc': LuCalendarArrowDown,
    'date-desc': LuCalendarArrowUp,
    'page-asc': SortDownIcon,
    'page-desc': SortUpIcon,
};

export function AnnotationSortButton({
    order,
    onToggle,
}: AnnotationSortButtonProps) {
    const Icon = SORT_ORDER_ICONS[order];

    return (
        <Button
            variant="neutral"
            icon={Icon}
            aria-label={translation.get(SORT_ORDER_LABELS[order])}
            onClick={onToggle}
        />
    );
}
