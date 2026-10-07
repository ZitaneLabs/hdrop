import { expect, test } from '@playwright/test'

test('encoded paths cannot redirect to another origin', async ({ page, request, baseURL }) => {
    for (const path of ['/%5Credirect.invalid/', '/%2F%5Credirect.invalid/']) {
        // Check without following redirects before letting the browser navigate.
        const response = await request.get(path, { maxRedirects: 0 })
        expect(response.headers().location).toBeUndefined()
        expect(response.status()).toBe(404)
        await page.goto(path)
        expect(new URL(page.url()).origin).toBe(new URL(baseURL!).origin)
    }
})

test('static upload and token URLs preserve encryption, navigation and browser-only passwords', async ({ page, baseURL }) => {
    const token = 'abcde12345'
    const fileName = 'private-test.txt'
    const contents = 'Private browser-encrypted file contents.'
    let uploaded: FormData | undefined
    const requests: string[] = []
    const errors: string[] = []
    page.on('request', request => requests.push(`${request.url()}\n${request.postData() || ''}\n${JSON.stringify(request.headers())}`))
    page.on('pageerror', error => errors.push(error.message))
    page.on('console', message => {
        if (message.type() === 'error') errors.push(message.text())
    })

    // Exercise real browser crypto and HTTP serialization against a fixture API
    // that stores and returns the ciphertext produced by the upload page.
    await page.route('**/v1/**', async route => {
        const request = route.request()
        const path = new URL(request.url()).pathname
        if (path === '/v1/files' && request.method() === 'POST') {
            uploaded = await new Response(new Uint8Array(request.postDataBuffer()!), {
                headers: { 'Content-Type': request.headers()['content-type'] },
            }).formData()
            expect([...uploaded.keys()].sort()).toEqual([
                'challenge_data', 'challenge_hash', 'file_data', 'file_name_data', 'iv', 'salt',
            ])
            expect(request.postData()).not.toContain(contents)
            expect(request.postData()).not.toContain(fileName)
            await route.fulfill({ json: { access_token: token, update_token: '12345678' } })
        } else if (path === `/v1/files/${token}/challenge`) {
            expect(uploaded).toBeDefined()
            if (request.method() === 'GET') {
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
    expect(password.length).toBeGreaterThan(32)
    await page.getByText('Copy Link only', { exact: true }).click()
    const link = await page.evaluate(() => navigator.clipboard.readText())
    expect(link).toBe(`${baseURL}/${token}`)

    await page.goto(`${link}#${password}`)
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)
    await expect(page.getByRole('link', { name: 'Download file' })).toHaveAttribute('download', fileName)
    expect(page.url()).toBe(`${link}#${password}`)
    const downloadEvent = page.waitForEvent('download')
    await page.getByRole('link', { name: 'Download file' }).click()
    const download = await downloadEvent
    expect(download.suggestedFilename()).toBe(fileName)
    const chunks: Buffer[] = []
    for await (const chunk of await download.createReadStream()) chunks.push(Buffer.from(chunk))
    expect(Buffer.concat(chunks).toString()).toBe(contents)
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
    const challengeCount = requests.filter(request => request.includes(`/v1/files/${token}/challenge`)).length
    await page.goto(`${link}/`)
    await expect(page.getByPlaceholder('Password')).toBeVisible()
    expect(requests.filter(request => request.includes(`/v1/files/${token}/challenge`))).toHaveLength(challengeCount)
    await page.getByPlaceholder('Password').fill(password)
    await page.getByPlaceholder('Password').press('Enter')
    await expect(page.locator('textarea[readonly]')).toHaveValue(contents)

    for (const request of requests) {
        expect(request).not.toContain(password)
        expect(request).not.toContain(encodeURIComponent(password))
    }
    expect(errors).toEqual([])
})
