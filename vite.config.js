import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';

export default defineConfig({
    plugins: [vue()],
    base: './',
    build: {
        outDir: 'dist', 
        target: 'esnext',
        minify: false
    },
    server: {
        port: 5173,
        strictPort: true,
    }
});
