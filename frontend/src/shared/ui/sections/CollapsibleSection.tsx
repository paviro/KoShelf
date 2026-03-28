import type { ReactNode } from 'react';
import { LuChevronDown } from 'react-icons/lu';

import { translation } from '../../i18n';
import { Button } from '../button/Button';

type ToggleButtonProps = {
    visible: boolean;
    onClick: () => void;
};

function ToggleButton({ visible, onClick }: ToggleButtonProps) {
    return (
        <Button
            variant="neutral"
            icon={LuChevronDown}
            iconPosition="end"
            label={
                visible
                    ? translation.get('toggle.hide')
                    : translation.get('toggle.show')
            }
            className={`[&>svg]:transition-transform [&>svg]:duration-200 ${visible ? '' : '[&>svg]:-rotate-90'}`}
            data-section-toggle-button
            onClick={onClick}
        />
    );
}

type CollapsibleSectionProps = {
    sectionKey: string;
    accentClass: string;
    title: string;
    titleBadge?: ReactNode;
    defaultVisible?: boolean;
    visible: boolean;
    onToggle: () => void;
    controls?: ReactNode;
    controlsClassName?: string;
    contentClassName?: string;
    children: ReactNode;
};

export function CollapsibleSection({
    sectionKey,
    accentClass,
    title,
    titleBadge,
    defaultVisible = true,
    visible,
    onToggle,
    controls,
    controlsClassName = 'space-x-3',
    contentClassName,
    children,
}: CollapsibleSectionProps) {
    const containerClassName = [contentClassName, visible ? '' : 'hidden']
        .filter(Boolean)
        .join(' ');

    return (
        <section
            data-name={sectionKey}
            data-default-visible={defaultVisible ? 'true' : 'false'}
        >
            <div className="flex items-center justify-between gap-3 mb-4 md:mb-6 pb-4 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center space-x-3">
                    <div
                        className={`w-2 h-6 md:h-8 rounded-full ${accentClass}`}
                    ></div>
                    <h2 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                        {title}
                    </h2>
                    {titleBadge}
                </div>
                <div className={`flex items-center ${controlsClassName}`}>
                    {controls}
                    <ToggleButton visible={visible} onClick={onToggle} />
                </div>
            </div>

            <div id={`${sectionKey}Container`} className={containerClassName}>
                {children}
            </div>
        </section>
    );
}
