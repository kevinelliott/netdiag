import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		host: '0.0.0.0', // Listen on all interfaces for mobile dev
		port: 5173,
		strictPort: true, // Fail if port is in use instead of trying another
	}
});
