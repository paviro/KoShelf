import { useCallback, useMemo } from 'react';
import {
    LuDownload,
    LuImage,
    LuShare2,
    LuSmartphone,
    LuSquare,
} from 'react-icons/lu';
import type { IconType } from 'react-icons';

import { ApiHttpError } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import {
    Button,
    buttonVariants,
    type ButtonColor,
} from '../../../shared/ui/button/Button';
import { CloseButton } from '../../../shared/ui/button/CloseButton';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';
import type { CompletionsShareAssets } from '../api/recap-data';

type RecapShareVariant = 'story' | 'square' | 'banner';

type RecapShareModalProps = {
    open: boolean;
    onClose: () => void;
    year: number;
    shareAssets: CompletionsShareAssets | null;
};

type ShareOption = {
    variant: RecapShareVariant;
    titleKey: string;
    detailKey: string;
    icon: IconType;
    iconContainerClassName: string;
    iconClassName: string;
    buttonColor: ButtonColor;
    webpUrl: string;
};

function isMobileDevice(): boolean {
    return (
        /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
            navigator.userAgent,
        ) || navigator.maxTouchPoints > 2
    );
}

function canUseWebShare(): boolean {
    return (
        typeof navigator.share === 'function' &&
        typeof navigator.canShare === 'function'
    );
}

function buildFilename(
    year: number,
    variant: RecapShareVariant,
    ext: 'webp' | 'svg',
): string {
    return `koshelf_${year}_${variant}.${ext}`;
}

function deriveSvgUrl(webpUrl: string): string {
    return webpUrl.replace(/\.webp($|\?)/, '.svg$1');
}

export function RecapShareModal({
    open,
    onClose,
    year,
    shareAssets,
}: RecapShareModalProps) {
    const useWebShare = useMemo(() => isMobileDevice() && canUseWebShare(), []);

    const options = useMemo<ShareOption[]>(() => {
        if (!shareAssets) {
            return [];
        }

        return [
            {
                variant: 'story',
                titleKey: 'recap-story',
                detailKey: 'recap-story.details',
                icon: LuSmartphone,
                iconContainerClassName:
                    'bg-purple-500/20 dark:bg-linear-to-br dark:from-purple-500 dark:to-purple-600',
                iconClassName: 'text-purple-600 dark:text-white',
                buttonColor: 'purple',
                webpUrl: shareAssets.story_url,
            },
            {
                variant: 'square',
                titleKey: 'recap-square',
                detailKey: 'recap-square.details',
                icon: LuSquare,
                iconContainerClassName:
                    'bg-blue-500/20 dark:bg-linear-to-br dark:from-blue-500 dark:to-blue-600',
                iconClassName: 'text-blue-600 dark:text-white',
                buttonColor: 'blue',
                webpUrl: shareAssets.square_url,
            },
            {
                variant: 'banner',
                titleKey: 'recap-banner',
                detailKey: 'recap-banner.details',
                icon: LuImage,
                iconContainerClassName:
                    'bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600',
                iconClassName: 'text-green-600 dark:text-white',
                buttonColor: 'green',
                webpUrl: shareAssets.banner_url,
            },
        ];
    }, [shareAssets]);

    const triggerDownload = useCallback((url: string, filename: string) => {
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.download = filename;
        document.body.appendChild(anchor);
        anchor.click();
        document.body.removeChild(anchor);
    }, []);

    const handlePrimaryAction = useCallback(
        async (option: ShareOption) => {
            const webpFilename = buildFilename(year, option.variant, 'webp');

            if (!useWebShare) {
                triggerDownload(option.webpUrl, webpFilename);
                return;
            }

            try {
                const response = await fetch(option.webpUrl);
                if (!response.ok) {
                    throw new ApiHttpError(option.webpUrl, response.status);
                }

                const blob = await response.blob();
                const file = new File([blob], webpFilename, {
                    type: 'image/webp',
                });
                if (navigator.canShare({ files: [file] })) {
                    await navigator.share({
                        files: [file],
                        title: translation.get('my-reading-recap'),
                        text: `📚 My ${year} reading journey! These graphics were crafted by KoShelf, my KoReader reading companion. Check it out: https://github.com/paviro/KoShelf`,
                    });
                    return;
                }
            } catch (error) {
                if (error instanceof Error && error.name === 'AbortError') {
                    return;
                }
            }

            triggerDownload(option.webpUrl, webpFilename);
        },
        [triggerDownload, useWebShare, year],
    );

    if (!shareAssets) {
        return null;
    }

    const modalTitle = useWebShare
        ? translation.get('share.recap-label')
        : translation.get('download.recap-label');
    const primaryLabel = useWebShare
        ? translation.get('share')
        : translation.get('download');
    const primaryIcon = useWebShare ? LuShare2 : LuDownload;

    return (
        <ModalShell
            open={open}
            onClose={onClose}
            cardClassName="max-w-md w-full max-h-[90vh] overflow-y-auto bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="flex items-center justify-between p-4 border-b border-gray-200/70 dark:border-dark-700/50">
                <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                    {modalTitle}
                </h3>
                <CloseButton onClick={onClose} className="w-8 h-8 rounded-lg" />
            </div>

            <div className="p-4 space-y-3">
                {options.map((option) => {
                    const Icon = option.icon;
                    const svgFilename = buildFilename(
                        year,
                        option.variant,
                        'svg',
                    );
                    const webpFilename = buildFilename(
                        year,
                        option.variant,
                        'webp',
                    );
                    const svgUrl = deriveSvgUrl(option.webpUrl);

                    return (
                        <div
                            key={option.variant}
                            className="bg-white dark:bg-dark-850/80 border border-gray-200/70 dark:border-dark-700/50 rounded-xl p-4 shadow-xs"
                        >
                            <div className="flex items-center gap-3 mb-3">
                                <div
                                    className={`w-10 h-10 rounded-lg ${option.iconContainerClassName} flex items-center justify-center shrink-0`}
                                >
                                    <Icon
                                        className={`w-5 h-5 ${option.iconClassName}`}
                                        aria-hidden
                                    />
                                </div>
                                <div className="flex-1">
                                    <div className="font-semibold text-gray-900 dark:text-white">
                                        {translation.get(option.titleKey)}
                                    </div>
                                    <div className="text-xs font-medium text-gray-500 dark:text-gray-400">
                                        {translation.get(option.detailKey)}
                                    </div>
                                </div>
                            </div>
                            <div className="flex gap-2">
                                <Button
                                    color={option.buttonColor}
                                    icon={primaryIcon}
                                    className="flex-1"
                                    onClick={() =>
                                        void handlePrimaryAction(option)
                                    }
                                    title={`${primaryLabel} ${webpFilename}`}
                                    aria-label={`${primaryLabel} ${webpFilename}`}
                                >
                                    {primaryLabel}
                                </Button>
                                <a
                                    className={buttonVariants({
                                        color: option.buttonColor,
                                    })}
                                    href={svgUrl}
                                    download={svgFilename}
                                >
                                    SVG
                                </a>
                            </div>
                        </div>
                    );
                })}
            </div>
        </ModalShell>
    );
}
