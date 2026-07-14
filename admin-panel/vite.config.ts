import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	envDir: '../',
	plugins: [
		tailwindcss(),
		sveltekit()
	],
	server: {
		port: 5175,
		strictPort: false
	}
});
