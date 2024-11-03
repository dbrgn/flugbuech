import js from '@eslint/js';
import ts from 'typescript-eslint';
import {type TSESLint} from '@typescript-eslint/utils';
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import globals from 'globals';

/** Files that should be ignored globally by eslint. */
const IGNORES = [
    // Temporary files, generated files, dependencies
    '**/.DS_Store',
    '**/node_modules',
    'build',
    '.svelte-kit',
    'package',
    '**/.env',
    '**/.env.*',
    '!**/.env.example',
    '**/package-lock.json',

    // Certain third party libs
    'static/js/elements.flagmeister.min.js',
    'static/js/maplibre-gl.js',
    'static/js/*.component.js',
];

/** Files that should be ignored globally by typescript-eslint. */
const IGNORES_TS = ['**/*.js', '**/*.mjs', '**/*.cjs'];

/**
 * Apply the {@link IGNORES_TS} ignore to the specified config array.
 */
function typescriptOnly(
    configArray: TSESLint.FlatConfig.ConfigArray,
): TSESLint.FlatConfig.ConfigArray {
    return configArray.map((config) => ({
        ...config,
        ignores: IGNORES_TS,
    }));
}

export default ts.config(
    // Ignore certain files
    {ignores: IGNORES},

    // Apply JS base config
    js.configs.recommended,

    // Apply TS base config (but not to plain-JS files)
    ...typescriptOnly(ts.configs.recommendedTypeChecked),
    //...ts.configs.strictTypeChecked, // TODO
    //...ts.configs.stylisticTypeChecked, // TODO

    // Apply Svelte config
    ...svelte.configs['flat/prettier'],

    // Configure language options
    {
        languageOptions: {
            ecmaVersion: 2023,
            globals: {
                ...globals.browser,
                ...globals.node,
            },
            parserOptions: {
                projectService: true,
                extraFileExtensions: ['.svelte'],
            },
        },
    },
    {
        files: ['**/*.svelte'],
        languageOptions: {
            parser: svelteParser,
            ecmaVersion: 2023,
            sourceType: 'script',
            parserOptions: {
                parser: '@typescript-eslint/parser',
            },
        },
    },

    // Configure general rules
    {
        rules: {
            // Creates a lot of false positives, e.g. when throwing redirect()
            '@typescript-eslint/only-throw-error': 'off',
            // Disallows putting unknown caught errors in messages
            '@typescript-eslint/restrict-template-expressions': 'off',
        },
    },

    // Configure test rules
    {
        rules: {
            '@typescript-eslint/no-unused-expressions': 'off',
        },
        files: ['**/*.test.ts'],
    },

    // Configure SvelteKit rules
    {
        rules: {
            // Lots of false positives, see https://github.com/sveltejs/eslint-plugin-svelte/issues/413
            '@typescript-eslint/no-unsafe-call': 'off',
            '@typescript-eslint/no-unsafe-member-access': 'off',
        },
        files: ['**/+page.svelte'],
    },
);
