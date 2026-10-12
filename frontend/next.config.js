const { PHASE_DEVELOPMENT_SERVER } = require('next/constants')

/** @type {import('next').NextConfig | ((phase: string) => import('next').NextConfig)} */
module.exports = (phase) => {
    if (phase === PHASE_DEVELOPMENT_SERVER || process.env.HDROP_WEB_RUNTIME === 'nextjs') {
        return {
            output: 'standalone',
            skipTrailingSlashRedirect: true,
            async rewrites() {
                // Use the same download URLs in dev, Next.js and static hosting.
                return [{ source: '/:accessToken([0-9a-f]{5,64})', destination: '/download' }]
            },
        }
    }

    return { output: 'export' }
}
