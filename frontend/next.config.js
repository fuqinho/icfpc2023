const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  webpack: (config) => {
    config.experiments.asyncWebAssembly = true;
    config.plugins.push(
      new WasmPackPlugin({
        crateDirectory: path.resolve(__dirname, "../wasm"),
        forceMode: "production",
      }),
    );
    return config;
  },
  images: {
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
