import { LuStar } from 'react-icons/lu';

type StarRatingInputProps = {
    value: number;
    onChange: (rating: number) => void;
    disabled?: boolean;
    size?: 'sm' | 'md';
};

export function StarRatingInput({
    value,
    onChange,
    disabled = false,
    size = 'sm',
}: StarRatingInputProps) {
    const normalizedValue =
        typeof value === 'number' && Number.isFinite(value)
            ? Math.max(0, Math.min(5, Math.floor(value)))
            : 0;

    const starSize = size === 'md' ? 'w-7 h-7' : 'w-5 h-5';

    if (disabled) {
        return (
            <div className="flex items-center space-x-1">
                {Array.from({ length: 5 }, (_, index) => (
                    <LuStar
                        key={index}
                        className={`${starSize} ${
                            index < normalizedValue
                                ? 'text-yellow-400 fill-yellow-400'
                                : 'text-gray-300 dark:text-dark-500'
                        }`}
                        aria-hidden="true"
                    />
                ))}
            </div>
        );
    }

    return (
        <div
            className="flex items-center select-none"
            role="radiogroup"
            aria-label="Rating"
            onMouseDown={(e) => e.preventDefault()}
        >
            <div
                className="w-3 self-stretch cursor-pointer"
                onClick={() => onChange(0)}
                role="radio"
                aria-checked={normalizedValue === 0}
                aria-label="0 stars"
                tabIndex={0}
                onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        onChange(0);
                    }
                }}
            />
            {Array.from({ length: 5 }, (_, index) => {
                const starValue = index + 1;
                const filled = index < normalizedValue;

                return (
                    <div
                        key={index}
                        className="p-0.5 cursor-pointer"
                        onClick={() => onChange(starValue === normalizedValue ? 0 : starValue)}
                        role="radio"
                        aria-checked={starValue === normalizedValue}
                        aria-label={`${starValue} star${starValue !== 1 ? 's' : ''}`}
                        tabIndex={0}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                onChange(starValue);
                            }
                        }}
                    >
                        <LuStar
                            className={`${starSize} transition-colors ${
                                filled
                                    ? 'text-yellow-400 fill-yellow-400'
                                    : 'text-gray-300 dark:text-dark-500'
                            }`}
                            aria-hidden="true"
                        />
                    </div>
                );
            })}
        </div>
    );
}
