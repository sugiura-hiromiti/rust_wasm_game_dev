// webpack.config.js
const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = {
  mode: "development",
  entry: "./js/index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "main.js",
    publicPath: "auto",
    clean: true,
  },
  experiments: {
    asyncWebAssembly: true,
    topLevelAwait: true,
  },
  module: {
    rules: [{ test: /\.wasm$/, type: "webassembly/async" }],
  },
  resolve: { extensions: [".js", ".wasm"] },
  devServer: {
    static: path.join(__dirname, "static"),
    hot: true,
    port: 8081,
  },
  devtool: "source-map",
  plugins: [
    new WasmPackPlugin({
      // Where Cargo.toml lives:
      crateDirectory: path.resolve(__dirname),
      // Optional but nice to be explicit:
      outDir: path.resolve(__dirname, "pkg"),
      // Bundler glue is what Webpack wants:
      extraArgs: "--target bundler",
      // Use dev or release matching webpack mode:
      forceMode: "development",
    }),
    new CopyWebpackPlugin([
      { from: path.resolve(__dirname, "static"), to: path.resolve(__dirname, "dist") },
    ]),
  ],
};
