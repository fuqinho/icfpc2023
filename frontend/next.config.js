const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  webpack: (config, options) => {
    config.experiments.asyncWebAssembly = true;

    // https://github.com/vercel/next.js/issues/25852
    config.optimization.moduleIds = 'named';
    if (options.isServer) {
        config.output.webassemblyModuleFilename = './../static/wasm/[modulehash].wasm';
    } else {
        config.output.webassemblyModuleFilename = 'static/wasm/[modulehash].wasm';
    }

    config.plugins.push(
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../wasm"),
        forceMode: "production",
      }),
    );
    return config;
  },
  images: {
    loader: "custom",
    loaderFile: "./components/image_loader.js",
    remotePatterns: [
      {
        protocol: "https",
        hostname: "icfpc2023-backend-uadsges7eq-an.a.run.app",
      },
      {
        protocol: "https",
        hostname: "icfpc2023-data-uvjbiongouirno.storage.googleapis.com",
      },
    ],
  },
};

module.exports = nextConfig;
