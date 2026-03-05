import { LuStar } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

type LibraryReviewSectionProps = {
    note: string;
    rating: number | null;
    visible: boolean;
    onToggle: () => void;
};

export function LibraryReviewSection({
    note,
    rating,
    visible,
    onToggle,
}: LibraryReviewSectionProps) {
    const normalizedRating =
        typeof rating === 'number' && Number.isFinite(rating)
            ? Math.max(0, Math.min(5, Math.floor(rating)))
            : 0;

    return (
        <CollapsibleSection
            sectionKey="review"
            defaultVisible
            accentClass="bg-gradient-to-b from-green-400 to-green-600"
            title={translation.get('my-review')}
            visible={visible}
            onToggle={onToggle}
            controlsClassName="space-x-4"
            contentClassName="mb-8"
            controls={
                normalizedRating > 0 ? (
                    <div className="flex items-center space-x-1">
                        {Array.from({ length: 5 }, (_, index) => {
                            const filled = index < normalizedRating;

                            return (
                                <LuStar
                                    key={index}
                                    className={`w-5 h-5 ${
                                        filled
                                            ? 'text-yellow-400 fill-yellow-400'
                                            : 'text-gray-300 dark:text-dark-500'
                                    }`}
                                    aria-hidden="true"
                                />
                            );
                        })}
                    </div>
                ) : null
            }
        >
            <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                <div className="relative">
                    <div className="absolute top-0 left-0 w-1 h-full bg-gradient-to-b from-green-400 to-green-600 rounded-full"></div>
                    <div className="pl-6">
                        <p className="text-gray-700 dark:text-dark-300 leading-relaxed text-lg whitespace-pre-wrap">
                            {note}
                        </p>
                    </div>
                </div>
            </div>
        </CollapsibleSection>
    );
}
