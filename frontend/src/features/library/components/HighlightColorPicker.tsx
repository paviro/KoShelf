import { LuCheck } from 'react-icons/lu';

import { OverlayPicker } from './OverlayPicker';

const COLORS = [
    { name: 'yellow', bgClass: 'bg-yellow-500 dark:bg-yellow-400' },
    { name: 'red', bgClass: 'bg-red-500 dark:bg-red-400' },
    { name: 'orange', bgClass: 'bg-orange-500 dark:bg-orange-400' },
    { name: 'green', bgClass: 'bg-emerald-500 dark:bg-emerald-400' },
    { name: 'olive', bgClass: 'bg-lime-500 dark:bg-lime-400' },
    { name: 'cyan', bgClass: 'bg-cyan-500 dark:bg-cyan-400' },
    { name: 'blue', bgClass: 'bg-blue-500 dark:bg-blue-400' },
    { name: 'purple', bgClass: 'bg-purple-500 dark:bg-purple-400' },
    { name: 'gray', bgClass: 'bg-gray-500 dark:bg-gray-400' },
] as const;

type HighlightColorPickerProps = {
    anchorRef: React.RefObject<HTMLElement | null>;
    currentColor: string;
    onSelect: (color: string) => void;
    onClose: () => void;
};

export function HighlightColorPicker({
    anchorRef,
    currentColor,
    onSelect,
    onClose,
}: HighlightColorPickerProps) {
    return (
        <OverlayPicker anchorRef={anchorRef} onClose={onClose}>
            <div className="grid grid-cols-5 gap-1.5">
                {COLORS.map((color) => {
                    const isSelected = color.name === currentColor;

                    return (
                        <button
                            key={color.name}
                            type="button"
                            onClick={() => onSelect(color.name)}
                            className={`w-7 h-7 rounded-full flex items-center justify-center border-2 transition-transform hover:scale-110 ${color.bgClass}`}
                            style={{
                                borderColor: isSelected
                                    ? 'currentColor'
                                    : 'transparent',
                            }}
                            title={color.name}
                            aria-label={color.name}
                        >
                            {isSelected && (
                                <LuCheck className="w-3.5 h-3.5 text-white drop-shadow-sm" />
                            )}
                        </button>
                    );
                })}
            </div>
        </OverlayPicker>
    );
}
