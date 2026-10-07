function loadClient(webBase?: string, apiBase?: string) {
    jest.resetModules()
    if (webBase === undefined) delete process.env.NEXT_PUBLIC_WEB_BASE_URL
    else process.env.NEXT_PUBLIC_WEB_BASE_URL = webBase
    if (apiBase === undefined) delete process.env.NEXT_PUBLIC_API_BASE_URL
    else process.env.NEXT_PUBLIC_API_BASE_URL = apiBase
    return (jest.requireActual('./ApiClient') as typeof import('./ApiClient')).default
}

const originalWebBase = process.env.NEXT_PUBLIC_WEB_BASE_URL
const originalApiBase = process.env.NEXT_PUBLIC_API_BASE_URL

afterEach(() => {
    loadClient(originalWebBase, originalApiBase)
})

test('an unconfigured image uses the browser origin and same-origin API', () => {
    const client = loadClient()
    expect(client.buildEndpoint(['v1', 'files', 'abcde', 'challenge'])).toBe('/v1/files/abcde/challenge?')
    const link = new URL(client.getDownloadLink('abcde', 'browser-only-password'))
    expect(link.origin).toBe(window.location.origin)
    expect(link.pathname).toBe('/abcde')
    expect(link.search).toBe('')
    expect(link.hash).toBe('#browser-only-password')
    expect(client.getDownloadLink('abcde')).toBe(`${window.location.origin}/abcde`)
})

test('explicit build-time origins still work, including trailing slashes', () => {
    const client = loadClient('https://files.example/', 'https://api.example/')
    expect(client.buildEndpoint('/v1/files', { key: 'a b' })).toBe('https://api.example/v1/files?key=a+b')
    expect(client.getDownloadLink('abcde', 'secret')).toBe('https://files.example/abcde#secret')
})

test('empty build arguments use the same defaults as omitted arguments', () => {
    const client = loadClient('', '')
    expect(client.buildEndpoint('/status')).toBe('/status?')
    expect(client.getDownloadLink('abcde')).toBe(`${window.location.origin}/abcde`)
})
