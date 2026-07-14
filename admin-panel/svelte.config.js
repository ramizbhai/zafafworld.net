import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: vitePreprocess(),
    kit: {
        adapter: adapter(),
        alias: {
            '@zafaf/domain': './src/lib/domain',
            '@zafaf/api-contracts': './src/lib/api-contracts',
            '@zafaf/utils': './src/lib/utils'
        }
    }
};

export default config;
