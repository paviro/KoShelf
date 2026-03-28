import { LuCheck } from 'react-icons/lu';

import { HIGHLIGHT_COLORS } from '../lib/highlight-constants';
import { DropdownPortal } from '../../../shared/ui/dropdown/DropdownPortal';
import { PICKER_PANEL_BASE_CLASSNAME } from '../../../shared/ui/dropdown/dropdown-styles';

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
        <DropdownPortal
            triggerRef={anchorRef}
            open
            onClose={onClose}
            className={PICKER_PANEL_BASE_CLASSNAME}
            alignment={{ bottom: 'end', top: 'end', left: 'center' }}
            placements={['bottom', 'top', 'left']}
            gap={4}
        >
            <div className="grid grid-cols-5 gap-1.5">
                {HIGHLIGHT_COLORS.map((color) => {
                    const isSelected = color.name === currentColor;

                    return (
                        <button
                            key={color.name}
                            type="button"
                            onClick={() => onSelect(color.name)}
                            className={`w-7 h-7 rounded-full flex items-center justify-center border-2 transition-transform hover:scale-110 ${color.swatchClass}`}
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
        </DropdownPortal>
    );
}
