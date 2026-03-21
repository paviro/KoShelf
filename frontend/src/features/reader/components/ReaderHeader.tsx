import { LuArrowLeft, LuList } from 'react-icons/lu';
import { Link } from 'react-router';

import { translation } from '../../../shared/i18n';
import type {
    ReaderModeControl,
    ReaderStyleControl,
    ReaderToggleControl,
} from '../hooks/useReaderStyle';
import { ReaderSettingsPanel } from './ReaderSettingsPanel';

type ReaderHeaderProps = {
    title: string;
    chapterLabel: string;
    backHref: string;
    onBackClick: (event: React.MouseEvent<HTMLAnchorElement>) => void;
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
    wordSpacing: ReaderStyleControl;
    leftMargin: ReaderStyleControl;
    rightMargin: ReaderStyleControl;
    topMargin: ReaderStyleControl;
    bottomMargin: ReaderStyleControl;
    hyphenation: ReaderModeControl;
    floatingPunctuation: ReaderModeControl;
    embeddedFonts: ReaderToggleControl;
    onResetBookDefaults: () => void;
    canResetBookDefaults: boolean;
    onResetKoShelfDefaults: () => void;
    canResetKoShelfDefaults: boolean;
    hasDistinctBookDefaults: boolean;
    onDrawerOpen: () => void;
};

export function ReaderHeader({
    title,
    chapterLabel,
    backHref,
    onBackClick,
    fontSize,
    lineSpacing,
    wordSpacing,
    leftMargin,
    rightMargin,
    topMargin,
    bottomMargin,
    hyphenation,
    floatingPunctuation,
    embeddedFonts,
    onResetBookDefaults,
    canResetBookDefaults,
    onResetKoShelfDefaults,
    canResetKoShelfDefaults,
    hasDistinctBookDefaults,
    onDrawerOpen,
}: ReaderHeaderProps) {
    return (
        <header className="relative z-10 flex items-center justify-between h-[70px] md:h-[80px] px-4 md:px-6 border-b border-gray-200/50 dark:border-dark-700/50 bg-white/90 dark:bg-dark-950/75 backdrop-blur-xs shrink-0">
            <div className="flex items-center space-x-3 min-w-0 flex-1">
                <Link
                    to={backHref}
                    onClick={onBackClick}
                    className="flex items-center space-x-2 text-primary-400 hover:text-primary-300 transition-colors cursor-pointer shrink-0"
                    aria-label={translation.get('go-back.aria-label')}
                >
                    <LuArrowLeft className="w-6 h-6" aria-hidden="true" />
                </Link>

                <div className="h-8 w-px bg-gray-200 dark:bg-dark-700 mx-3 md:mx-6"></div>

                <div className="min-w-0 flex-1">
                    <h1 className="text-lg md:text-2xl font-bold text-gray-900 dark:text-white truncate">
                        {title}
                    </h1>
                    {chapterLabel && (
                        <p className="text-xs text-gray-500 dark:text-dark-300 truncate">
                            {chapterLabel}
                        </p>
                    )}
                </div>
            </div>

            <div className="flex items-center space-x-2 shrink-0 ml-3">
                <ReaderSettingsPanel
                    fontSize={fontSize}
                    lineSpacing={lineSpacing}
                    wordSpacing={wordSpacing}
                    leftMargin={leftMargin}
                    rightMargin={rightMargin}
                    topMargin={topMargin}
                    bottomMargin={bottomMargin}
                    hyphenation={hyphenation}
                    floatingPunctuation={floatingPunctuation}
                    embeddedFonts={embeddedFonts}
                    onResetBookDefaults={onResetBookDefaults}
                    canResetBookDefaults={canResetBookDefaults}
                    onResetKoShelfDefaults={onResetKoShelfDefaults}
                    canResetKoShelfDefaults={canResetKoShelfDefaults}
                    hasDistinctBookDefaults={hasDistinctBookDefaults}
                />

                <button
                    type="button"
                    onClick={onDrawerOpen}
                    className="flex items-center justify-center w-10 h-10 p-2.5 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors duration-200 backdrop-blur-xs focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                    aria-label={translation.get('reader-drawer-aria')}
                >
                    <LuList className="w-5 h-5" aria-hidden="true" />
                </button>
            </div>
        </header>
    );
}
