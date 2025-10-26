/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'export',
  distDir: 'out',
  images: {
    unoptimized: true, // Required for static export
  },
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || '/api/v1',
  },
  trailingSlash: true, // Helps with static serving
  // Skip dynamic route validation for static export
  experimental: {
    missingSuspenseWithCSRBailout: false,
  },
  webpack: (config) => {
    // Monaco editor worker setup
    config.module.rules.push({
      test: /\.worker\.js$/,
      use: { loader: 'worker-loader' },
    })
    return config
  },
}

module.exports = nextConfig
