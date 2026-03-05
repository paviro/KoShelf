import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig(function (_a) {
    var mode = _a.mode;
    var env = loadEnv(mode, process.cwd(), '');
    var backendTarget = env.KOSHELF_DEV_BACKEND_URL || 'http://localhost:3000';
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
            sourcemap: false,
            emptyOutDir: true,
            rollupOptions: {
                output: {
                    entryFileNames: 'assets/js/[name]-[hash].js',
                    chunkFileNames: 'assets/js/[name]-[hash].js',
                    assetFileNames: function (assetInfo) {
                        var _a;
                        if ((_a = assetInfo.name) === null || _a === void 0 ? void 0 : _a.endsWith('.css')) {
                            return 'assets/css/[name]-[hash][extname]';
                        }
                        return 'assets/[name]-[hash][extname]';
                    },
                },
            },
        },
    };
});
