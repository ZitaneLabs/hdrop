import { expect, test } from '@playwright/test'

const site = process.env.HDROP_TEST_URL || ''

test('a deployed VPS can upload, decrypt and delete a file', async ({ page, context }) => {
    test.skip(!site, 'Set HDROP_TEST_URL to the HTTPS deployment to check')
    const contents = 'hdrop VPS installation check'
    await page.goto(site)
    await page.locator('input[type=file]').setInputFiles({
        name: 'vps-check.txt', mimeType: 'text/plain', buffer: Buffer.from(contents),
    })
    const remove = page.getByText('Delete', { exact: true })
    await expect(remove).toBeVisible()
    try {
        await page.getByText('Copy Password', { exact: true }).click()
        const password = await page.evaluate(() => navigator.clipboard.readText())
        await page.getByText('Copy Link only', { exact: true }).click()
        const link = await page.evaluate(() => navigator.clipboard.readText())
        expect(new URL(link).origin).toBe(new URL(site).origin)
        const recipient = await context.newPage()
        await recipient.goto(`${link}#${password}`)
        await expect(recipient.locator('textarea[readonly]')).toHaveValue(contents)
        await expect(recipient.getByRole('link', { name: 'Download file' })).toHaveAttribute('download', 'vps-check.txt')
    } finally {
        const deleted = page.waitForResponse(response => response.request().method() === 'DELETE')
        await remove.click()
        expect((await deleted).ok()).toBe(true)
    }
})
