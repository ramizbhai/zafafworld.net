import { sveltekit } from '@sveltejs/kit/vite';
import { paraglide } from '@inlang/paraglide-sveltekit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig, createLogger } from 'vite';
import path from 'path';
import fs from 'fs';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

let paraglideAdapterUtilsPath = '';
try {
	const sveltekitPath = require.resolve('@inlang/paraglide-sveltekit');
	const sveltekitDir = path.resolve(path.dirname(sveltekitPath), '../..');
	const nestedPath = path.resolve(sveltekitDir, 'node_modules/@inlang/paraglide-js/dist/adapter-utils/index.js');
	const hoistedPath = path.resolve(sveltekitDir, '../@inlang/paraglide-js/dist/adapter-utils/index.js');
	if (fs.existsSync(nestedPath)) {
		paraglideAdapterUtilsPath = nestedPath;
	} else if (fs.existsSync(hoistedPath)) {
		paraglideAdapterUtilsPath = hoistedPath;
	}
} catch (e) {
	paraglideAdapterUtilsPath = path.resolve(process.cwd(),
		'./node_modules/@inlang/paraglide-sveltekit/node_modules/@inlang/paraglide-js/dist/adapter-utils/index.js'
	);
}

const SUPPRESS = [
	'no loadMessages in resolved Modules found',
	'No plugin provides a loadMessages',
	'forgot to add a plugin',
	'noise.png referenced in',
	'node_modules/@inlang/paraglide-unplugin/node_modules/@inlang/sdk/dist/resolve-modules/import.js',
	'plugin.api.sveltePreprocess'
];

const originalStdoutWrite = process.stdout.write.bind(process.stdout);
// @ts-ignore
process.stdout.write = function (chunk: any, encoding: any, callback: any) {
	const str = typeof chunk === 'string' ? chunk : chunk.toString();
	if (SUPPRESS.some(s => str.includes(s))) {
		if (typeof encoding === 'function') encoding();
		else if (typeof callback === 'function') callback();
		return true;
	}
	return originalStdoutWrite(chunk, encoding, callback);
};

const originalStderrWrite = process.stderr.write.bind(process.stderr);
// @ts-ignore
process.stderr.write = function (chunk: any, encoding: any, callback: any) {
	const str = typeof chunk === 'string' ? chunk : chunk.toString();
	if (SUPPRESS.some(s => str.includes(s))) {
		if (typeof encoding === 'function') encoding();
		else if (typeof callback === 'function') callback();
		return true;
	}
	return originalStderrWrite(chunk, encoding, callback);
};

const logger = createLogger();

const runtimePath = path.resolve(__dirname, './src/lib/paraglide/runtime.js');
function hasExport(content: string, name: string): boolean {
	const pattern = new RegExp(`\\bexport\\s+(const|let|var|function|class)\\s+${name}\\b|\\bexport\\s*\\{[^}]*\\b${name}\\b[^}]*\\}`);
	return pattern.test(content);
}

function patchRuntime() {
	if (fs.existsSync(runtimePath)) {
		try {
			const code = fs.readFileSync(runtimePath, 'utf-8');
			let extra = '';
			if (!hasExport(code, 'getLocale')) {
				extra += '\nexport const getLocale = languageTag;';
			}
			if (!hasExport(code, 'baseLocale')) {
				extra += '\nexport const baseLocale = sourceLanguageTag;';
			}
			if (!hasExport(code, 'locales')) {
				extra += '\nexport const locales = availableLanguageTags;';
			}
			if (extra.length > 0) {
				fs.writeFileSync(runtimePath, code + extra + '\n', 'utf-8');
				console.log('[patch-paraglide-runtime] Patched runtime.js on disk.');
			}
		} catch (e) {
			console.error('[patch-paraglide-runtime] Failed to patch runtime.js:', e);
		}
	}
}

// Run patch immediately on config load
patchRuntime();

export default defineConfig({
	envDir: '../',
	plugins: [
		paraglide({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			disablePreprocessor: false
		}),
		{
			name: 'patch-paraglide-runtime',
			buildStart() {
				patchRuntime();
			},
			transform(code, id) {
				if (id.endsWith('paraglide/runtime.js')) {
					let extra = '';
					if (!hasExport(code, 'getLocale')) {
						extra += '\nexport const getLocale = languageTag;';
					}
					if (!hasExport(code, 'baseLocale')) {
						extra += '\nexport const baseLocale = sourceLanguageTag;';
					}
					if (!hasExport(code, 'locales')) {
						extra += '\nexport const locales = availableLanguageTags;';
					}
					if (extra.length > 0) {
						try {
							fs.writeFileSync(id, code + extra + '\n', 'utf-8');
						} catch (e) {}
						return {
							code: code + extra + '\n',
							map: null
						};
					}
				}
			}
		},

		tailwindcss(),
		sveltekit()
	],
	resolve: {
		alias: {
			'@inlang/paraglide-js/internal/adapter-utils': paraglideAdapterUtilsPath
		}
	},
	ssr: {
		noExternal: ['@inlang/paraglide-sveltekit', '@inlang/paraglide-js']
	},
	customLogger: {
		...logger,
		warn(msg, options) {
			if (SUPPRESS.some(s => msg.includes(s))) return;
			logger.warn(msg, options);
		}
	}
});