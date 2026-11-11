import { expect, test } from '@playwright/test'

test('serves pages, token URLs and cached assets without redirecting unknown paths', async ({ request }) => {
    const isStatic = (process.env.FRONTEND_RUNTIME || 'static') === 'static'
    const shell = await (await request.get('/download')).text()
    expect((await request.get('/favicon.ico')).status()).toBe(200)
    for (const path of ['/', '/privacy', '/download', '/abcde', '/abcde/', `/${'a'.repeat(64)}`, `/${'a'.repeat(64)}/`]) {
        const response = await request.get(path, { maxRedirects: 0 })
        expect(response.status()).toBe(200)
        expect(response.headers()['content-type']).toContain('text/html')
        if (isStatic) {
            expect(response.headers()['cache-control']).toBe('no-cache')
            if (/^\/[0-9a-f]{5,64}\/?$/.test(path)) expect(await response.text()).toBe(shell)
        }
    }
    for (const path of [
        '/abcd', `/${'a'.repeat(65)}`, '/abcde/nested', '/missing.js', '/_next/static/missing.js',
        '/not-a-page/', '/status', '/v1/files', '/%5Credirect.invalid/', '/%2F%5Credirect.invalid/',
    ]) {
        const response = await request.get(path, { maxRedirects: 0 })
        expect(response.status()).toBe(404)
        expect(response.headers().location).toBeUndefined()
        if (isStatic) expect(response.headers()['cache-control']).toBe('no-cache')
    }
    const home = await request.get('/')
    const asset = (await home.text()).match(/\/_next\/static\/[^" ]+\.js/)?.[0]
    expect(asset).toBeDefined()
    const response = await request.get(asset!)
    expect(response.status()).toBe(200)
    expect(response.headers()['cache-control']).toBe('public, max-age=31536000, immutable')

    if (isStatic) {
        for (const path of ['/privacy', '/download']) {
            const response = await request.get(`${path}/?source=slash-test`, { maxRedirects: 0 })
            expect(response.status()).toBe(301)
            expect(response.headers().location).toBe(`${path}?source=slash-test`)
            expect((await request.get(`${path}/`)).status()).toBe(200)
        }
        const compressed = await request.get('/', { headers: { 'Accept-Encoding': 'gzip' } })
        expect(compressed.headers()['content-encoding']).toBe('gzip')
        expect(compressed.headers().vary).toContain('Accept-Encoding')
    }
})

test('frontend upload and token URLs preserve encryption, navigation and browser-only passwords', async ({ page, baseURL }) => {
    const token = 'abcde'
    const fileName = 'private-test.txt'
    const contents = 'Private browser-encrypted file contents.'
    let uploaded: FormData | undefined
    let challengeRequests = 0
    const requests: string[] = []
    const errors: string[] = []
    page.on('request', request => requests.push(`${request.url()}\n${request.postData() || ''}\n${JSON.stringify(request.headers())}`))
    page.on('pageerror', error => errors.push(error.message))

    // Return the uploaded ciphertext so successful decryption checks the URL password.
    await page.route('**/v1/**', async route => {
        const request = route.request()
        expect(new URL(request.url()).origin).toBe(new URL(baseURL!).origin)
        const path = new URL(request.url()).pathname
        if (path === '/v1/files' && request.method() === 'POST') {
            uploaded = await new Response(new Uint8Array(request.postDataBuffer()!), {
                headers: { 'Content-Type': request.headers()['content-type'] },
            }).formData()
            await route.fulfill({ json: { access_token: token, update_token: '12345678' } })
        } else if (path === `/v1/files/${token}/challenge`) {
            if (request.method() === 'GET') {
                challengeRequests++
                await route.fulfill({ json: {
                    iv: uploaded!.get('iv'),
                    salt: uploaded!.get('salt'),
                    challenge: uploaded!.get('challenge_data'),
                } })
            } else {
                expect(request.postDataJSON()).toEqual({ challenge: uploaded!.get('challenge_hash') })
                await route.fulfill({ json: { file_name_data: uploaded!.get('file_name_data') } })
            }
        } else if (path === `/v1/files/${token}` && request.method() === 'GET') {
            expect(request.headers().authorization).toBe(`Bearer ${uploaded!.get('challenge_hash')}`)
            await route.fulfill({
                contentType: 'application/octet-stream',
                body: Buffer.from(await (uploaded!.get('file_data') as File).arrayBuffer()),
            })
        } else {
            throw new Error(`Unexpected API request: ${request.method()} ${path}`)
        }
    })

    await page.goto('/')
    await page.locator('input[type=file]').setInputFiles({ name: fileName, mimeType: 'text/plain', buffer: Buffer.from(contents) })
    await page.getByText('Copy Password', { exact: true }).click()
    const password = await page.evaluate(() => navigator.clipboard.readText())
    await page.getByText('Copy Link only', { exact: true }).click()
    const link = await page.evaluate(() => navigator.clipboard.readText())
    expect(link).toBe(`${baseURL}/${token}`)

    await page.goto(`/download#${password}`)
    await expect(page.getByText('Invalid download link.', { exact: true })).toBeVisible()
    expect(challengeRequests).toBe(0)

    await page.goto(`${link}#${password}`)
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)
    expect(challengeRequests).toBe(1)
    await expect(page.getByRole('link', { name: 'Download file' })).toHaveAttribute('download', fileName)
    expect(page.url()).toBe(`${link}#${password}`)
    await page.reload()
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)

    await page.getByRole('link', { name: 'Privacy', exact: true }).click()
    await expect(page).toHaveURL(`${baseURL}/privacy`)
    await expect(page.getByRole('heading', { name: 'Open Source' })).toBeVisible()
    await page.goBack()
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)
    await page.getByRole('link', { name: 'Upload', exact: true }).click()
    await expect(page.locator('input[type=file]')).toBeAttached()

    // A link without a fragment waits for a manually entered password.
    const challengeCount = challengeRequests
    await page.goto(`${link}/`)
    await expect(page.getByPlaceholder('Password')).toBeVisible()
    expect(challengeRequests).toBe(challengeCount)
    await page.getByPlaceholder('Password').fill(password)
    await page.getByPlaceholder('Password').press('Enter')
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)
    expect(challengeRequests).toBe(challengeCount + 1)
    expect(page.url()).toBe(`${link}/`)

    for (const request of requests) {
        expect(request).not.toContain(password)
        expect(request).not.toContain(encodeURIComponent(password))
    }
    expect(errors).toEqual([])
})
