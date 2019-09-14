const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  target: "electron-renderer",
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--out-name index"
    }),
  ]
};
