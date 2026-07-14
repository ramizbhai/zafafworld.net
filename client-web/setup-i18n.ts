import fs from 'fs';
import path from 'path';
import { ar } from './src/lib/i18n/translations/ar.ts';
import { en } from './src/lib/i18n/translations/en.ts';

function flattenObj(obj, prefix = '') {
  return Object.keys(obj).reduce((acc, k) => {
    const pre = prefix.length ? prefix + '_' : '';
    if (typeof obj[k] === 'object' && obj[k] !== null && !Array.isArray(obj[k])) {
      Object.assign(acc, flattenObj(obj[k], pre + k));
    } else {
      acc[pre + k] = obj[k];
    }
    return acc;
  }, {});
}

// Ensure directories exist
fs.mkdirSync('./messages', { recursive: true });
fs.mkdirSync('./project.inlang', { recursive: true });

// Write messages
fs.writeFileSync('./messages/ar.json', JSON.stringify(flattenObj(ar), null, 2));
fs.writeFileSync('./messages/en.json', JSON.stringify(flattenObj(en), null, 2));

// Write project settings
const settings = {
  "$schema": "https://inlang.com/schema/project-settings",
  "sourceLanguageTag": "ar",
  "languageTags": ["ar", "en"],
  "modules": [
    "https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js",
    "https://cdn.jsdelivr.net/npm/@inlang/plugin-m-function-matcher@latest/dist/index.js"
  ],
  "plugin.inlang.messageFormat": {
    "pathPattern": "./messages/{languageTag}.json"
  }
};
fs.writeFileSync('./project.inlang/settings.json', JSON.stringify(settings, null, 2));

console.log('Setup complete.');
