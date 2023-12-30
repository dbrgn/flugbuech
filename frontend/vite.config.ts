import {sveltekit} from '@sveltejs/kit/vite';
import {defineConfig} from 'vitest/config';

export default defineConfig({
    plugins: [sveltekit()],
    test: {
        include: ['src/**/*.{test,spec}.{js,ts}'],
    },
    server: {
        fs: {
            allow: ['..'],
        },
        // To avoid CORS-requests, proxy requests to the API (running on a different port). In
        // production, a reverse proxy will ensure that both the frontend and the API will run on
        // the same domain/port.
        proxy: {
            '/api/v1': {
                target: 'http://127.0.0.1:8000',
            },
        },
    },
});
