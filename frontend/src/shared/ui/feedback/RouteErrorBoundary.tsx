import { Component } from 'react';
import type { ErrorInfo, ReactNode } from 'react';
import { LuExternalLink, LuRotateCw, LuTriangleAlert } from 'react-icons/lu';

import { translation } from '../../i18n';
import { Button } from '../button/Button';
import { buttonVariants } from '../button/button-variants';

const ISSUES_URL = 'https://github.com/paviro/KoShelf/issues/new';
const RETRY_COOLDOWN_MS = 600;

type RouteErrorBoundaryProps = {
    children: ReactNode;
};

type RouteErrorBoundaryState = {
    hasError: boolean;
    error: Error | null;
    hasRetried: boolean;
    retrying: boolean;
};

function buildIssueUrl(error: Error | null): string {
    const title = encodeURIComponent(
        `[Bug] Rendering crash: ${error?.message ?? 'Unknown error'}`,
    );
    const body = encodeURIComponent(
        [
            '## Description',
            'The app crashed with a rendering error.',
            '',
            '## Error',
            `**Message:** ${error?.message ?? 'Unknown error'}`,
            '```',
            error?.stack ?? 'No stack trace available',
            '```',
            '',
            '## Steps to reproduce',
            '1. ',
            '',
            `**URL:** \`${window.location.href}\``,
            `**User-Agent:** \`${navigator.userAgent}\``,
        ].join('\n'),
    );

    return `${ISSUES_URL}?title=${title}&body=${body}`;
}

export class RouteErrorBoundary extends Component<
    RouteErrorBoundaryProps,
    RouteErrorBoundaryState
> {
    private retryTimer: number | null = null;

    constructor(props: RouteErrorBoundaryProps) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            hasRetried: false,
            retrying: false,
        };
    }

    static getDerivedStateFromError(
        error: Error,
    ): Partial<RouteErrorBoundaryState> {
        return { hasError: true, error, retrying: false };
    }

    componentDidCatch(error: Error, info: ErrorInfo): void {
        console.error('RouteErrorBoundary caught an error:', error, info);
    }

    componentWillUnmount(): void {
        if (this.retryTimer !== null) {
            window.clearTimeout(this.retryTimer);
        }
    }

    private handleRetry = (): void => {
        if (this.state.retrying) return;

        this.setState({ retrying: true });

        this.retryTimer = window.setTimeout(() => {
            this.retryTimer = null;
            this.setState({
                hasError: false,
                error: null,
                hasRetried: true,
                retrying: false,
            });
        }, RETRY_COOLDOWN_MS);
    };

    render() {
        if (!this.state.hasError) {
            return this.props.children;
        }

        const { error, hasRetried, retrying } = this.state;
        const errorMessage = error?.message ?? 'Unknown error';

        return (
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
                <div className="w-full max-w-md bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-6 text-center">
                    <div className="mx-auto mb-4 w-14 h-14 bg-red-500/20 dark:bg-linear-to-br dark:from-red-500 dark:to-rose-500 rounded-xl flex items-center justify-center">
                        <LuTriangleAlert
                            className="w-7 h-7 text-red-600 dark:text-white"
                            aria-hidden
                        />
                    </div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                        {translation.get('error-state.crash-title')}
                    </h3>
                    <p className="text-sm font-medium text-gray-600 dark:text-dark-300 leading-relaxed mb-4">
                        {translation.get('error-state.crash-description')}
                    </p>

                    <pre className="mb-6 text-left text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950/30 border border-red-200/50 dark:border-red-800/30 rounded-lg p-3 overflow-auto max-h-32 whitespace-pre-wrap break-words">
                        {errorMessage}
                    </pre>

                    <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
                        <Button onClick={this.handleRetry} disabled={retrying}>
                            <LuRotateCw
                                className={`w-4 h-4 ${retrying ? 'animate-spin' : ''}`}
                                aria-hidden
                            />
                            {translation.get('error-state.retry')}
                        </Button>
                        {hasRetried && (
                            <a
                                href={buildIssueUrl(error)}
                                target="_blank"
                                rel="noopener noreferrer"
                                className={buttonVariants({
                                    color: 'secondary',
                                })}
                            >
                                <LuExternalLink
                                    className="w-4 h-4"
                                    aria-hidden
                                />
                                {translation.get('error-state.crash-report')}
                            </a>
                        )}
                    </div>
                </div>
            </div>
        );
    }
}
