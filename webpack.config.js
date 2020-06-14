const path = require('path');
const CopyPlugin = require('copy-webpack-plugin');

const mode = process.env.NODE_ENV || 'development';
const prod = mode === 'production';

module.exports = {
    entry: {
        map: ['./static/svelte/map.js'],
    },
    externals: {
        'mapbox-gl': 'mapboxgl',
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
                use: {
                    loader: 'svelte-loader',
                    options: {}
                }
            }
        ]
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                {
                    from: path.resolve('node_modules', 'mapbox-gl', 'dist', 'mapbox-gl.css'),
                    to: __dirname + '/static/css/'
                },
                {
                    from: path.resolve('node_modules', 'mapbox-gl', 'dist', 'mapbox-gl.js'),
                    to: __dirname + '/static/js/'
                },
                {
                    from: path.resolve('node_modules', 'mapbox-gl', 'dist', 'mapbox-gl.js.map'),
                    to: __dirname + '/static/js/'
                },
            ],
        }),
    ],
    performance: {
        assetFilter: function(assetFilename) {
            return !(/\.map$/.test(assetFilename))
                && !assetFilename.startsWith('mapbox-gl.');
        },
    },
};
