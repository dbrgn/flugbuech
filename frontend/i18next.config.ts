import {defineConfig, type Plugin} from 'i18next-cli';
import {parse, type AST} from 'svelte/compiler';

/**
 * Plugin i18next-cli-plugin-svelte taken from
 * https://github.com/dreamscached/i18next-cli-plugin-svelte/:
 *
 * MIT License
 *
 * Copyright (c) 2025 Herman S.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
const i18nextSveltePlugin: Plugin = {
    name: 'i18next-cli-plugin-svelte',
    onLoad: (code: string, path: string) => {
        // Passthrough for non-Svelte files
        if (!path.match(/\.svelte$/)) return code;

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const fromAst = (node: any) => code.slice(node.content.start, node.content.end);

        const ast = parse(code, {filename: path}) as AST.Root;
        const extracted: string[] = [];
        if (ast.instance) extracted.push(fromAst(ast.instance));
        if (ast.module) extracted.push(fromAst(ast.module));

        console.log('extracted ' + extracted);

        // When contatenating make sure we don't cause issues with ASI
        return extracted.join('\n;');
    },
};

export default defineConfig({
    locales: ['de', 'en'],
    extract: {
        input: 'src/**/*.{ts,svelte}',
        output: 'src/translations/{{language}}/{{namespace}}.json',
    },
    types: {
        input: ['src/translations/{{language}}/{{namespace}}.json'],
        output: 'src/types/i18next.d.ts',
    },
    plugins: [i18nextSveltePlugin],
});
