import { LuCheck } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { DRAWER_TYPES } from '../lib/highlight-constants';
import { OverlayPicker } from './OverlayPicker';

type HighlightDrawerPickerProps = {
    anchorRef: React.RefObject<HTMLElement | null>;
    currentDrawer: string;
    onSelect: (drawer: string) => void;
    onClose: () => void;
};

export function HighlightDrawerPicker({
    anchorRef,
    currentDrawer,
    onSelect,
    onClose,
}: HighlightDrawerPickerProps) {
    return (
        <OverlayPicker anchorRef={anchorRef} onClose={onClose}>
            <div className="flex flex-col gap-0.5">
                {DRAWER_TYPES.map((drawer) => {
                    const isSelected = drawer.name === currentDrawer;
                    const Icon = drawer.icon;

                    return (
                        <button
                            key={drawer.name}
                            type="button"
                            onClick={() => onSelect(drawer.name)}
                            className={`flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-sm transition-colors ${
                                isSelected
                                    ? 'bg-primary-500/15 text-primary-700 dark:text-primary-300'
                                    : 'text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/60'
                            }`}
                        >
                            <Icon
                                className="w-4 h-4 shrink-0"
                                aria-hidden="true"
                            />
                            <span className={drawer.sampleClass}>
                                {translation.get(drawer.labelKey)}
                            </span>
                            {isSelected && (
                                <LuCheck className="w-3.5 h-3.5 ml-auto text-primary-600 dark:text-primary-400" />
                            )}
                        </button>
                    );
                })}
            </div>
        </OverlayPicker>
    );
}
