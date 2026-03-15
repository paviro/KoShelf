import { LuArrowUpRight, LuBuilding2, LuHash } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import type { ExternalIdentifier } from '../api/library-data';

type LibraryAdditionalInfoSectionProps = {
    publisher: string | null;
    identifiers: ExternalIdentifier[];
    visible: boolean;
    onToggle: () => void;
};

export function LibraryAdditionalInfoSection({
    publisher,
    identifiers,
    visible,
    onToggle,
}: LibraryAdditionalInfoSectionProps) {
    return (
        <CollapsibleSection
            sectionKey="additional-info"
            defaultVisible={false}
            accentClass="bg-linear-to-b from-cyan-400 to-cyan-600"
            title={translation.get('additional-information')}
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
        >
            <div className="space-y-6">
                {publisher !== null && (
                    <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
                        <div className="flex items-center space-x-3">
                            <div className="w-10 h-10 bg-indigo-500/20 dark:bg-linear-to-br dark:from-indigo-500 dark:to-indigo-600 rounded-lg flex items-center justify-center">
                                <LuBuilding2
                                    className="w-5 h-5 text-indigo-600 dark:text-white"
                                    aria-hidden="true"
                                />
                            </div>
                            <div>
                                <div className="text-lg font-bold text-gray-900 dark:text-white">
                                    {publisher}
                                </div>
                                <div className="text-sm text-gray-500 dark:text-dark-400">
                                    {translation.get('publisher')}
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {identifiers.length > 0 && (
                    <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-6">
                        <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
                            <div className="w-8 h-8 bg-purple-500/20 dark:bg-linear-to-br dark:from-purple-500 dark:to-purple-600 rounded-lg flex items-center justify-center mr-3">
                                <LuHash
                                    className="w-4 h-4 text-purple-600 dark:text-white"
                                    aria-hidden="true"
                                />
                            </div>
                            {translation.get('book-identifiers')}
                        </h4>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            {identifiers.map((identifier) => {
                                const key = `${identifier.scheme}:${identifier.value}`;
                                const linkable = Boolean(identifier.url);

                                if (linkable) {
                                    return (
                                        <a
                                            key={key}
                                            href={identifier.url ?? undefined}
                                            target="_blank"
                                            rel="noreferrer"
                                            className="group relative bg-gray-100 dark:bg-dark-700 border border-gray-300 dark:border-dark-600 rounded-lg p-4 hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-dark-650 transition-all duration-200 shadow-xs hover:shadow-md"
                                        >
                                            <div className="flex items-center justify-between mb-2">
                                                <div className="text-sm font-medium text-primary-600 dark:text-primary-300 uppercase tracking-wide group-hover:text-primary-700 dark:group-hover:text-primary-200">
                                                    {identifier.display_scheme}
                                                </div>
                                                <div className="text-gray-400 dark:text-dark-400 transform translate-x-1 group-hover:translate-x-0 transition-all duration-200">
                                                    <LuArrowUpRight
                                                        className="w-4 h-4"
                                                        aria-hidden="true"
                                                    />
                                                </div>
                                            </div>
                                            <div className="text-sm text-gray-700 dark:text-dark-300 font-mono break-all group-hover:text-gray-900 dark:group-hover:text-white transition-colors duration-200">
                                                {identifier.value}
                                            </div>
                                        </a>
                                    );
                                }

                                return (
                                    <div
                                        key={key}
                                        className="bg-gray-100 dark:bg-dark-700 border border-gray-300 dark:border-dark-600 rounded-lg p-4 shadow-xs"
                                    >
                                        <div className="text-sm font-medium text-primary-600 dark:text-primary-300 uppercase tracking-wide mb-2">
                                            {identifier.display_scheme}
                                        </div>
                                        <div className="text-sm text-gray-700 dark:text-dark-300 font-mono break-all">
                                            {identifier.value}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                )}
            </div>
        </CollapsibleSection>
    );
}
