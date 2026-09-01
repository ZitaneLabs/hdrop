import { act } from 'react'
import { createRoot } from 'react-dom/client'

jest.mock('mime/lite', () => ({
    __esModule: true,
    default: { getType: () => 'application/octet-stream' },
}))

import FilePreview from './FilePreview'

globalThis.IS_REACT_ACT_ENVIRONMENT = true

test('revokes object URLs when preview data changes and unmounts', async () => {
    const createObjectURL = jest
        .fn()
        .mockReturnValueOnce('blob:first')
        .mockReturnValueOnce('blob:second')
    const revokeObjectURL = jest.fn()
    Object.defineProperties(URL, {
        createObjectURL: { configurable: true, value: createObjectURL },
        revokeObjectURL: { configurable: true, value: revokeObjectURL },
    })

    const root = createRoot(document.createElement('div'))
    await act(() => root.render(
        <FilePreview data={new Uint8Array([1, 2, 3, 4]).buffer} fileName="first.png" />
    ))
    await act(() => root.render(
        <FilePreview data={new Uint8Array([5, 6, 7, 8]).buffer} fileName="second.png" />
    ))
    await act(() => root.unmount())

    expect(createObjectURL).toHaveBeenCalledTimes(2)
    expect(revokeObjectURL.mock.calls).toEqual([['blob:first'], ['blob:second']])
})
