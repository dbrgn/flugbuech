const path = require('path');
const CopyPlugin = require('copy-webpack-plugin');
const sveltePreprocess = require('svelte-preprocess');

const mode = process.env.NODE_ENV || 'development';
const prod = mode === 'production';

const sveltePreprocessOptions = {
    typescript: {},
};

module.exports = {
    entry: {
        'map': ['./static/svelte/map.js'],
        'password-change-form': ['./static/svelte/password-change-form.js'],
    },
    externals: {
        'maplibre-gl': 'maplibregl',
    },
    resolve: {
        alias: {
            svelte: path.resolve('node_modules', 'svelte')
        },
        extensions: ['.mjs', '.js', '.svelte'],
        mainFields: ['svelte', 'browser', 'module', 'main']
    },
    output: {
        path: __dirname + '/static/js',
        filename: '[name].component.js',
        chunkFilename: '[name].component.[id].js'
    },
    mode: mode,
    devtool: prod ? false: 'source-map',
    module: {
        rules: [
            {
                test: /\.svelte$/,
                exclude: /node_modules/,
                use: {
                    loader: 'svelte-loader',
                    options: {
                        preprocess: sveltePreprocess(sveltePreprocessOptions),
                    }
                }
            }
        ]
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                {
                    from: path.resolve('node_modules', 'maplibre-gl', 'dist', 'maplibre-gl.css'),
                    to: __dirname + '/static/css/'
                },
                {
                    from: path.resolve('node_modules', 'maplibre-gl', 'dist', 'maplibre-gl.js'),
                    to: __dirname + '/static/js/'
                },
                {
                    from: path.resolve('node_modules', 'maplibre-gl', 'dist', 'maplibre-gl.js.map'),
                    to: __dirname + '/static/js/'
                },
            ],
        }),
    ],
    performance: {
        assetFilter: function(assetFilename) {
            return !(/\.map$/.test(assetFilename))
                && !assetFilename.startsWith('maplibre-gl.');
        },
    },
};
