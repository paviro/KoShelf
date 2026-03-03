import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
    const env = loadEnv(mode, process.cwd(), '');
    const backendTarget = env.KOSHELF_DEV_BACKEND_URL || 'http://localhost:3000';

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
            },
        },
        build: {
            outDir: 'dist',
            assetsDir: 'react-assets',
            sourcemap: false,
            emptyOutDir: true,
        },
    };
});
