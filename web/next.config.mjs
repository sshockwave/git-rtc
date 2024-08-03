/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  async rewrites() {
    return [
      {
        source: '/ws',
        destination: 'http://localhost:9242/ws'
      }
    ]
  }
};

export default nextConfig;
