import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd(), '');
    const backendTarget =
        env.KOSHELF_DEV_BACKEND_URL || 'http://localhost:3000';

    return {
        plugins: [react()],
        server: {
            proxy: {
                '/api': {
                    target: backendTarget,
                    changeOrigin: true,
                },
                '/assets': {
                    target: backendTarget,
                    changeOrigin: true,
                },
                '/core': {
                    target: backendTarget,
                    changeOrigin: true,
                },
            },
        },
        build: {
            outDir: 'dist',
            sourcemap: false,
            emptyOutDir: true,
            rollupOptions: {
                output: {
                    entryFileNames: 'core/js/[name]-[hash].js',
                    chunkFileNames: 'core/js/[name]-[hash].js',
                    assetFileNames: (assetInfo) => {
                        const assetName = assetInfo.name ?? '';

                        if (assetName.endsWith('.css')) {
                            return 'core/css/[name]-[hash][extname]';
                        }

                        if (/\.(woff2?|ttf|otf|eot)$/i.test(assetName)) {
                            return 'core/fonts/[name]-[hash][extname]';
                        }

                        return 'core/[name]-[hash][extname]';
                    },
                },
            },
        },
    };
});
