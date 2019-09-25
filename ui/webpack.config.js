const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require("copy-webpack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./src/index.ts"
  },
  devtool: 'inline-source-map',
  module: {
    rules: [
      {
        test: /\.(ts|tsx)?$/,
        include: path.resolve(__dirname, './src'),
        use: [{
          loader: 'ts-loader'
        }],
      }
    ]
  },
  optimization: {
    splitChunks: {
        cacheGroups: {
            commons: {
                test: /[\\/]node_modules[\\/]/,
                name: "vendors",
                chunks: "all"
            }
        }
    }
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js']
  },
  devServer: {
    contentBase: dist,
  },
  output: {
    path: dist,
    filename: "bundle.js"
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "index.html")
    ]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./data-model-wasm"),
      extraArgs: "--out-name index"
    }),

  ]
};
