import {
    LuFileQuestion,
    LuRotateCw,
    LuTriangleAlert,
    LuWifiOff,
} from 'react-icons/lu';
import type { ReactNode } from 'react';

import { isApiHttpError } from '../../api';
import { translation } from '../../i18n';
import { Button } from '../button/Button';
import { PageStateLayout } from './PageStateLayout';

type ErrorVariant = 'generic' | 'not-found' | 'connection' | 'file-unavailable';

type PageErrorStateProps = {
    error?: unknown;
    onRetry?: () => void;
    children?: ReactNode;
    layout?: 'page' | 'overlay';
};

function resolveVariant(error: unknown): ErrorVariant {
    if (isApiHttpError(error)) {
        if (error.code === 'book_file_unavailable') {
            return 'file-unavailable';
        }
        if (error.status === 404) {
            return 'not-found';
        }
        return 'generic';
    }

    if (error instanceof TypeError) {
        return 'connection';
    }

    return 'generic';
}

const VARIANT_CONFIG: Record<
    ErrorVariant,
    {
        icon: typeof LuTriangleAlert;
        gradientFrom: string;
        gradientTo: string;
        glowFrom: string;
        glowTo: string;
        titleKey: string;
        descriptionKey: string;
    }
> = {
    generic: {
        icon: LuTriangleAlert,
        gradientFrom: 'from-red-500',
        gradientTo: 'to-rose-500',
        glowFrom: 'from-red-500/20',
        glowTo: 'to-rose-500/20',
        titleKey: 'error-state.title',
        descriptionKey: 'error-state.description',
    },
    'not-found': {
        icon: LuFileQuestion,
        gradientFrom: 'from-amber-500',
        gradientTo: 'to-orange-500',
        glowFrom: 'from-amber-500/20',
        glowTo: 'to-orange-500/20',
        titleKey: 'error-state.not-found-title',
        descriptionKey: 'error-state.not-found-description',
    },
    connection: {
        icon: LuWifiOff,
        gradientFrom: 'from-slate-500',
        gradientTo: 'to-gray-600',
        glowFrom: 'from-slate-500/20',
        glowTo: 'to-gray-600/20',
        titleKey: 'error-state.connection-title',
        descriptionKey: 'error-state.connection-description',
    },
    'file-unavailable': {
        icon: LuTriangleAlert,
        gradientFrom: 'from-amber-500',
        gradientTo: 'to-yellow-500',
        glowFrom: 'from-amber-500/20',
        glowTo: 'to-yellow-500/20',
        titleKey: 'error-state.file-unavailable-title',
        descriptionKey: 'error-state.file-unavailable-description',
    },
};

export function PageErrorState({
    error,
    onRetry,
    children,
    layout = 'page',
}: PageErrorStateProps) {
    const variant = resolveVariant(error);
    const config = VARIANT_CONFIG[variant];
    const Icon = config.icon;
    const hasActions = Boolean(onRetry || children);

    return (
        <PageStateLayout
            icon={<Icon className="w-12 h-12 text-white" aria-hidden />}
            gradientFrom={config.gradientFrom}
            gradientTo={config.gradientTo}
            glowFrom={config.glowFrom}
            glowTo={config.glowTo}
            title={translation.get(config.titleKey)}
            description={translation.get(config.descriptionKey)}
            layout={layout}
        >
            {hasActions && (
                <div className="flex flex-col sm:flex-row items-center gap-3 mt-6">
                    {onRetry && (
                        <Button onClick={onRetry}>
                            <LuRotateCw className="w-4 h-4" aria-hidden />
                            {translation.get('error-state.retry')}
                        </Button>
                    )}
                    {children}
                </div>
            )}
        </PageStateLayout>
    );
}
