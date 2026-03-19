import { LuArrowLeft, LuList, LuMinus, LuPlus } from 'react-icons/lu';
import { Link } from 'react-router';

import { translation } from '../../../shared/i18n';

type ReaderHeaderProps = {
    title: string;
    chapterLabel: string;
    backHref: string;
    onBackClick: (event: React.MouseEvent<HTMLAnchorElement>) => void;
    onFontDecrease: () => void;
    onFontIncrease: () => void;
    onDrawerOpen: () => void;
};

export function ReaderHeader({
    title,
    chapterLabel,
    backHref,
    onBackClick,
    onFontDecrease,
    onFontIncrease,
    onDrawerOpen,
}: ReaderHeaderProps) {
    return (
        <header className="flex items-center justify-between h-[70px] md:h-[80px] px-4 md:px-6 border-b border-gray-200/50 dark:border-dark-700/50 bg-white/90 dark:bg-dark-950/75 backdrop-blur-xs shrink-0">
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

            <div className="flex items-center space-x-1 shrink-0 ml-3">
                <button
                    type="button"
                    onClick={onFontDecrease}
                    className="flex items-center justify-center w-9 h-9 text-primary-400 hover:text-primary-300 transition-colors cursor-pointer"
                    aria-label="Decrease font size"
                >
                    <LuMinus className="w-5 h-5" aria-hidden="true" />
                </button>
                <button
                    type="button"
                    onClick={onFontIncrease}
                    className="flex items-center justify-center w-9 h-9 text-primary-400 hover:text-primary-300 transition-colors cursor-pointer"
                    aria-label="Increase font size"
                >
                    <LuPlus className="w-5 h-5" aria-hidden="true" />
                </button>

                <button
                    type="button"
                    onClick={onDrawerOpen}
                    className="flex items-center text-primary-400 hover:text-primary-300 transition-colors cursor-pointer"
                    aria-label={translation.get('reader-drawer-aria')}
                >
                    <LuList className="w-6 h-6" aria-hidden="true" />
                </button>
            </div>
        </header>
    );
}
