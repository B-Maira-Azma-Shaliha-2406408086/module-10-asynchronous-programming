const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const distPath = path.resolve(__dirname, 'dist');

module.exports = {
    mode: 'production',
    devServer: {
        port: 8000,
    },
    entry: {},
    output: {
        path: distPath,
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                { from: './static', to: distPath },
                { from: './pkg', to: path.resolve(distPath, 'pkg') },
            ],
        }),
    ],
};
