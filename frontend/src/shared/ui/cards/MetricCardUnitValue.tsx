import {
    joinUnitValueParts,
    type UnitValuePart,
} from '../../lib/intl/unit-value';

type MetricCardUnitValueProps = {
    value: UnitValuePart[];
    size?: 'default' | 'compact';
};

export function MetricCardUnitValue({
    value,
    size = 'default',
}: MetricCardUnitValueProps) {
    const parts = value.filter((part) => part.amount || part.unit);
    const hasUnits = parts.some((part) => Boolean(part.unit));

    if (!hasUnits) {
        return <>{joinUnitValueParts(parts)}</>;
    }

    const unitStyleClass =
        size === 'compact'
            ? 'text-xs text-gray-600 dark:text-gray-300'
            : 'text-base text-gray-500 dark:text-gray-400';

    return parts.map((part, index) => (
        <span key={`${part.amount}-${part.unit ?? ''}-${index}`}>
            <span className="whitespace-nowrap">
                <span>{part.amount}</span>
                {part.unit && (
                    <span
                        className={`${part.amount ? 'ml-0.5 ' : ''}align-baseline ${unitStyleClass} leading-none font-medium`}
                    >
                        {part.unit}
                    </span>
                )}
            </span>
            {index < parts.length - 1 ? ' ' : null}
        </span>
    ));
}
