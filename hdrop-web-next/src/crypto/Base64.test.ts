import Base64 from "./Base64"
import { TextEncoder } from 'util'

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

    test('encode long string', () => {
        // given
        const bytes = new TextEncoder().encode("Hello this is a test")
        // when
        const result = Base64.encode(bytes)
        // then
        expect(result).toBe('SGVsbG8gdGhpcyBpcyBhIHRlc3Q=')
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

    test('decode long string', () => {
        // given
        const str = 'SGVsbG8gdGhpcyBpcyBhIHRlc3Q='
        const expected = Array.from(new TextEncoder().encode("Hello this is a test"))
        // when
        const result = Array.from(Base64.decode(str))
        // then
        expect(result).toEqual(expected)
    })
})
