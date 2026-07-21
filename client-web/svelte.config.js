import adapter from '@sveltejs/adapter-node';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
		runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true)
	},
	kit: {
		csrf: {
			checkOrigin: false
		},
		// adapter-auto only supports some environments, see https://svelte.dev/docs/kit/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://svelte.dev/docs/kit/adapters for more information about adapters.
		adapter: adapter(),
		alias: {
			// Expose param matchers as importable modules from anywhere in the app.
			// This lets load functions and components import shared constants like
			// SORT_URL_TO_API and CITY_COUNTRY_MAP directly from the matcher files.
			'$params': 'src/params',
			'$params/*': 'src/params/*',
		}
	}
};

export default config;
