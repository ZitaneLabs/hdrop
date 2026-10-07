import { act } from 'react'
import { createRoot, Root } from 'react-dom/client'

jest.mock('@/api', () => ({ Downloader: { downloadFile: jest.fn().mockResolvedValue(undefined) } }))
jest.mock('react-hot-toast', () => ({ toast: { error: jest.fn() }, Toaster: () => null }))
jest.mock('react-wavify', () => ({ __esModule: true, default: () => null }))
jest.mock('@/components', () => {
    const { default: Switch, Match } = jest.requireActual('@/components/Switch')
    return {
        Switch,
        Match,
        PasswordField: jest.requireActual('@/components/PasswordField').default,
        FilePreview: () => null,
    }
})

import { Downloader } from '@/api'
import DownloadFilePage from './page'

Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true })

let container: HTMLDivElement
let root: Root

beforeEach(() => {
    jest.clearAllMocks()
    container = document.createElement('div')
    document.body.appendChild(container)
    root = createRoot(container)
})

afterEach(async () => {
    await act(() => root.unmount())
    container.remove()
    window.history.replaceState(null, '', '/')
})

test.each(['/abcde', '/0123456789abcdef/'])(
    'starts a download from the browser URL %s and its private fragment',
    async pathname => {
        window.history.replaceState(null, '', `${pathname}#browser-only-password`)

        await act(() => root.render(<DownloadFilePage />))

        expect(Downloader.downloadFile).toHaveBeenCalledTimes(1)
        expect(Downloader.downloadFile).toHaveBeenCalledWith(expect.objectContaining({
            accessToken: pathname.replaceAll('/', ''),
            password: 'browser-only-password',
        }))
        expect(window.location.pathname).toBe(pathname)
        expect(window.location.hash).toBe('#browser-only-password')
    },
)

test('waits for a manually entered password when the link has no fragment', async () => {
    window.history.replaceState(null, '', '/abcde')
    await act(() => root.render(<DownloadFilePage />))
    expect(Downloader.downloadFile).not.toHaveBeenCalled()

    const input = container.querySelector('input')!
    await act(() => {
        const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')!.set!
        setValue.call(input, 'entered-password')
        input.dispatchEvent(new Event('input', { bubbles: true }))
    })
    await act(() => {
        input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    })

    expect(Downloader.downloadFile).toHaveBeenCalledTimes(1)
    expect(Downloader.downloadFile).toHaveBeenCalledWith(expect.objectContaining({
        accessToken: 'abcde',
        password: 'entered-password',
    }))
    expect(window.location.href).not.toContain('entered-password')
})

test.each(['/download', '/privacy', '/abcde/extra', '/_next/missing.js', '/abcd', `/${'a'.repeat(65)}`])(
    'does not start a download for invalid token path %s',
    async pathname => {
        window.history.replaceState(null, '', `${pathname}#browser-only-password`)
        await act(() => root.render(<DownloadFilePage />))

        expect(Downloader.downloadFile).not.toHaveBeenCalled()
        expect(container.textContent).toContain('Invalid download link.')
    },
)
