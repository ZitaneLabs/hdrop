import { defineConfig } from '@playwright/test'

export default defineConfig({
    testDir: './e2e',
    workers: 1,
    use: {
        baseURL: process.env.STATIC_BASE_URL || 'http://127.0.0.1:8080',
        permissions: ['clipboard-read', 'clipboard-write'],
        trace: 'retain-on-failure',
    },
})
