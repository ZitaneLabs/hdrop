import Base64 from "./Base64"

describe('Base64', () => {
    test('encode', () => {
        // given
        const bytes = new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7])
        // when
        const result = Base64.encode(bytes)
        // then
        expect(result).toBe('AAECAwQFBgc=')
    })

    test('encode zero bytes', () => {
        // given
        const bytes = new Uint8Array([])
        // when
        const result = Base64.encode(bytes)
        // then
        expect(result).toBe('')
    })

    test('decode', () => {
        // given
        const str = 'AAECAwQFBgc='
        // when
        const result = Base64.decode(str)
        // then
        expect(result).toEqual(new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7]))
    })

    test('decode zero bytes', () => {
        // given
        const str = ''
        // when
        const result = Base64.decode(str)
        // then
        expect(result).toEqual(new Uint8Array([]))
    })
})
