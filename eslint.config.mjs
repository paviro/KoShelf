import js from '@eslint/js';
import eslintConfigPrettier from 'eslint-config-prettier';
import tseslint from 'typescript-eslint';

export default tseslint.config(
    js.configs.recommended,
    ...tseslint.configs.recommended,
    eslintConfigPrettier,
    {
        files: ['assets/ts/**/*.ts'],
        languageOptions: {
            parserOptions: {
                // Keep linting fast and non-type-aware by default.
                // (We already have `npm run typecheck` for the type-aware pass.)
                project: null,
                tsconfigRootDir: import.meta.dirname,
            },
        },
        rules: {
            // Prefer modern TS patterns / safer defaults
            '@typescript-eslint/consistent-type-imports': [
                'warn',
                { prefer: 'type-imports', fixStyle: 'separate-type-imports' },
            ],
            '@typescript-eslint/no-explicit-any': 'warn',
            '@typescript-eslint/no-non-null-assertion': 'warn',
            '@typescript-eslint/no-unused-vars': [
                'warn',
                {
                    argsIgnorePattern: '^_',
                    varsIgnorePattern: '^_',
                    caughtErrorsIgnorePattern: '^_',
                },
            ],

            // General hygiene
            'no-console': 'off',
            // State vars are sometimes intentionally reassigned later; `prefer-const` isn’t very helpful here.
            // (We can re-enable later if we want it repo-wide.)
            'prefer-const': 'off',
        },
    },
);

