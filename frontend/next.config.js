const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.experiments.asyncWebAssembly = true;
    config.plugins.push(
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../examples/lifegame-wasm"),
      }),
    );
    return config;
  },
};

module.exports = nextConfig;
