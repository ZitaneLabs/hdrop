const { PHASE_DEVELOPMENT_SERVER } = require('next/constants')

/** @type {import('next').NextConfig | ((phase: string) => import('next').NextConfig)} */
module.exports = (phase) => {
    if (phase === PHASE_DEVELOPMENT_SERVER) {
        return {
            async rewrites() {
                // Match the static server's download aliases during local development.
                return [{ source: '/:accessToken([0-9a-f]{5,64})', destination: '/download' }]
            },
        }
    }

    return { output: 'export' }
}
