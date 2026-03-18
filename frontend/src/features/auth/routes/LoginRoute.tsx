import { useCallback, useEffect, useState } from 'react';
import { Navigate, useNavigate } from 'react-router';

import { BRAND_ICON } from '../../../app/shell/shell-nav';
import { api, isApiHttpError } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { LoginPasswordField } from '../components/LoginPasswordField';
import { LoginSubmitButton } from '../components/LoginSubmitButton';

type LoginRouteProps = {
    defaultRoute: '/books' | '/comics' | '/statistics';
    siteTitle: string;
    authEnabled: boolean;
    authenticated: boolean;
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
    authenticated,
    siteLoaded,
}: LoginRouteProps) {
    const navigate = useNavigate();
    const [password, setPassword] = useState('');
    const [submitError, setSubmitError] = useState<string | null>(null);
    const [submitPending, setSubmitPending] = useState(false);
    const BrandIcon = BRAND_ICON;
    const loginTitle = translation.get('login-title', { site: siteTitle });

    useEffect(() => {
        document.title = loginTitle;
    }, [loginTitle]);

    useEffect(() => {
        if (siteLoaded && authEnabled && authenticated) {
            navigate(defaultRoute, { replace: true });
        }
    }, [authenticated, authEnabled, defaultRoute, navigate, siteLoaded]);

    if (!authEnabled) {
        return <Navigate to={defaultRoute} replace />;
    }

    const handleSubmit = useCallback(
        async (event: React.FormEvent<HTMLFormElement>) => {
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
        },
        [defaultRoute, navigate, password, submitPending],
    );

    return (
        <main className="min-h-dvh bg-gray-100 dark:bg-dark-925">
            <div className="mx-auto flex min-h-dvh w-full max-w-6xl items-center px-4 py-10 sm:px-6 lg:px-8">
                <section className="grid w-full overflow-hidden rounded-3xl border border-gray-200/70 dark:border-dark-700/70 bg-white/95 dark:bg-dark-900/90 shadow-2xl md:grid-cols-5">
                    <aside className="relative flex flex-col gap-8 overflow-hidden bg-linear-to-br from-primary-700 via-primary-600 to-primary-500 p-6 text-white sm:p-8 md:col-span-2 md:justify-between">
                        <div className="pointer-events-none absolute inset-0 bg-linear-to-br from-white/20 via-transparent to-transparent" />
                        <div className="pointer-events-none absolute -right-8 -bottom-8 md:-right-10 md:-bottom-10">
                            <BrandIcon
                                className="h-24 w-24 text-white/10 md:h-36 md:w-36"
                                aria-hidden="true"
                            />
                        </div>

                        <div className="relative inline-flex items-center gap-3 rounded-2xl border border-white/25 bg-white/10 px-3 py-3 shadow-lg shadow-primary-900/20">
                            <span className="flex h-10 w-10 items-center justify-center rounded-xl bg-white/20">
                                <BrandIcon
                                    className="h-5 w-5"
                                    aria-hidden="true"
                                />
                            </span>
                            <div>
                                <p className="text-lg font-semibold leading-tight">
                                    {siteTitle}
                                </p>
                            </div>
                        </div>

                        <div className="relative">
                            <p className="text-3xl font-semibold leading-tight text-white md:text-4xl">
                                {translation.get('reading-companion')}
                            </p>
                        </div>
                    </aside>

                    <div className="md:col-span-3 p-6 pb-8 sm:p-8 sm:pb-10 md:p-10 md:pb-12">
                        <header className="mb-6 space-y-3">
                            <h1 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white leading-tight">
                                {translation.get('login')}
                            </h1>
                        </header>

                        <form className="space-y-5" onSubmit={handleSubmit}>
                            <LoginPasswordField
                                password={password}
                                disabled={submitPending}
                                onPasswordChange={setPassword}
                            />

                            {submitError ? (
                                <p
                                    role="alert"
                                    className="text-sm rounded-lg border border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300 px-3 py-2"
                                >
                                    {submitError}
                                </p>
                            ) : null}

                            <LoginSubmitButton
                                disabled={
                                    submitPending ||
                                    password.trim().length === 0
                                }
                            />
                        </form>
                    </div>
                </section>
            </div>
        </main>
    );
}
