import { useEffect, useState } from 'react';
import { Navigate, useNavigate } from 'react-router';

import { api, isApiHttpError } from '../../../shared/api';
import { fetchJson } from '../../../shared/api-fetch';
import { translation } from '../../../shared/i18n';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';

type LoginRouteProps = {
    defaultRoute: '/books' | '/comics' | '/statistics';
    siteTitle: string;
    authEnabled: boolean;
    siteLoaded: boolean;
};

function resolveLoginErrorMessage(error: unknown): string {
    if (!isApiHttpError(error)) {
        return translation.get('error-state.connection-title');
    }

    if (error.status === 429) {
        return translation.get('login-rate-limited');
    }

    if (error.status === 401) {
        return translation.get('login-error');
    }

    return error.apiMessage ?? translation.get('error-state.title');
}

export function LoginRoute({
    defaultRoute,
    siteTitle,
    authEnabled,
    siteLoaded,
}: LoginRouteProps) {
    const navigate = useNavigate();
    const [password, setPassword] = useState('');
    const [submitError, setSubmitError] = useState<string | null>(null);
    const [submitPending, setSubmitPending] = useState(false);
    const [sessionCheckPending, setSessionCheckPending] = useState(true);
    const loginTitle = translation.get('login-title', { site: siteTitle });

    useEffect(() => {
        document.title = loginTitle;
    }, [loginTitle]);

    useEffect(() => {
        if (!siteLoaded) {
            setSessionCheckPending(true);
            return;
        }

        if (!authEnabled) {
            setSessionCheckPending(false);
            return;
        }

        let cancelled = false;

        const checkExistingSession = async () => {
            setSessionCheckPending(true);
            try {
                await fetchJson('/api/auth/sessions', {
                    redirectOnUnauthorized: false,
                });
                if (!cancelled) {
                    navigate(defaultRoute, { replace: true });
                }
            } catch (error) {
                if (
                    !cancelled &&
                    (!isApiHttpError(error) || error.status !== 401)
                ) {
                    setSubmitError(resolveLoginErrorMessage(error));
                }
            } finally {
                if (!cancelled) {
                    setSessionCheckPending(false);
                }
            }
        };

        void checkExistingSession();

        return () => {
            cancelled = true;
        };
    }, [authEnabled, defaultRoute, navigate, siteLoaded]);

    if (!siteLoaded || sessionCheckPending) {
        return (
            <main className="min-h-dvh flex items-center justify-center px-6 py-10">
                <LoadingSpinner
                    size="lg"
                    srLabel="Loading login"
                    delayMs={10}
                />
            </main>
        );
    }

    if (!authEnabled) {
        return <Navigate to={defaultRoute} replace />;
    }

    const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();

        if (submitPending) {
            return;
        }

        setSubmitError(null);
        setSubmitPending(true);

        try {
            await api.login(password);
            navigate(defaultRoute, { replace: true });
        } catch (error) {
            setSubmitError(resolveLoginErrorMessage(error));
        } finally {
            setSubmitPending(false);
        }
    };

    return (
        <main className="min-h-dvh flex items-center justify-center px-4 py-10 bg-linear-to-br from-sky-100 via-white to-emerald-100 dark:from-dark-950 dark:via-dark-900 dark:to-dark-850">
            <section className="w-full max-w-md">
                <div className="rounded-2xl border border-gray-200/70 dark:border-dark-700/70 bg-white/90 dark:bg-dark-900/85 backdrop-blur-sm shadow-xl p-6 md:p-8">
                    <header className="mb-6 space-y-2">
                        <p className="text-xs uppercase tracking-[0.22em] text-primary-700 dark:text-primary-300 font-semibold">
                            {translation.get('login')}
                        </p>
                        <h1 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white leading-tight">
                            {loginTitle}
                        </h1>
                    </header>

                    <form className="space-y-4" onSubmit={handleSubmit}>
                        <div className="space-y-2">
                            <label
                                htmlFor="login-password"
                                className="block text-sm font-medium text-gray-900 dark:text-white"
                            >
                                {translation.get('login-password')}
                            </label>
                            <input
                                id="login-password"
                                type="password"
                                autoComplete="current-password"
                                value={password}
                                onChange={(event) =>
                                    setPassword(event.target.value)
                                }
                                className="w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:outline-hidden focus:ring-2 focus:ring-primary-500/60"
                                disabled={submitPending}
                                required
                            />
                        </div>

                        {submitError ? (
                            <p
                                role="alert"
                                className="text-sm rounded-lg border border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300 px-3 py-2"
                            >
                                {submitError}
                            </p>
                        ) : null}

                        <button
                            type="submit"
                            className="w-full inline-flex items-center justify-center rounded-lg px-4 py-2.5 text-sm font-semibold text-white bg-primary-600 hover:bg-primary-500 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                            disabled={
                                submitPending || password.trim().length === 0
                            }
                        >
                            {translation.get('login-submit')}
                        </button>
                    </form>
                </div>
            </section>
        </main>
    );
}
