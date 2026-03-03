import { LuInfo, LuSearch } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';

export function LibraryEmptyState() {
    return (
        <section className="flex flex-col items-center justify-center text-center min-h-[calc(100vh_-_12.5rem)] md:min-h-[calc(100vh_-_13rem)] lg:min-h-[calc(100vh_-_7.5rem)] py-8">
            <div className="flex flex-col items-center justify-center -mt-8">
                <div className="relative mb-8">
                    <div className="absolute inset-0 w-32 h-32 bg-gradient-to-br from-amber-500/20 to-amber-600/20 rounded-full blur-2xl" />
                    <div className="relative w-24 h-24 bg-gradient-to-br from-amber-500 to-amber-600 rounded-2xl flex items-center justify-center shadow-2xl">
                        <LuSearch className="w-12 h-12 text-white" aria-hidden="true" />
                    </div>
                </div>

                <h3 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white mb-4">
                    {translation.get('no-books-found')}
                </h3>
                <p className="text-lg text-gray-600 dark:text-dark-300 max-w-2xl leading-relaxed">
                    {translation.get('no-books-match')}
                </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-4 items-center mt-8">
                <div className="flex items-center space-x-2 text-sm text-gray-500 dark:text-dark-400">
                    <LuInfo className="w-4 h-4" aria-hidden="true" />
                    <span>{translation.get('try-adjusting')}</span>
                </div>
            </div>
        </section>
    );
}
