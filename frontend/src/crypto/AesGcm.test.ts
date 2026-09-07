/** @jest-environment node */
import AesGcm, { FILE_AAD, FILE_NAME_AAD, CHALLENGE_AAD, FILE_NAME_XOR_MASK, CHALLENGE_XOR_MASK } from './AesGcm'

const domains = [FILE_AAD, FILE_NAME_AAD, CHALLENGE_AAD]
let key: CryptoKey

beforeAll(async () => {
    key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt'])
})

test('uses stable UTF-8 protocol domains', () => {
    expect(domains.map(aad => new TextDecoder().decode(aad))).toEqual([
        'hdrop/v1/file', 'hdrop/v1/filename', 'hdrop/v1/challenge',
    ])
})

describe.each(domains.map(aad => [new TextDecoder().decode(aad), aad] as const))('%s', (_, aad) => {
    test('round-trips with matching AAD and a 128-bit tag', async () => {
        const params = AesGcm.generateParams()
        const plaintext = new TextEncoder().encode('private data 🔒')
        const ciphertext = await AesGcm.encrypt(plaintext, key, params, aad)
        expect(ciphertext.byteLength).toBe(plaintext.byteLength + 16)
        expect(new Uint8Array(await AesGcm.decrypt(ciphertext, key, params, aad))).toEqual(plaintext)
    })

    test.each(domains.filter(other => other !== aad).map(other => [new TextDecoder().decode(other), other] as const))(
        'rejects substitution as %s even with the original IV', async (_, wrongAad) => {
            const params = AesGcm.generateParams()
            const ciphertext = await AesGcm.encrypt(new Uint8Array([1, 2, 3]), key, params, aad)
            await expect(AesGcm.decrypt(ciphertext, key, params, wrongAad)).rejects.toMatchObject({ name: 'OperationError' })
        },
    )
})

test('derives three distinct 96-bit IVs without changing the base IV', () => {
    const base = AesGcm.generateParams()
    const restored = AesGcm.restoreParams(new Uint8Array(base.iv as Uint8Array))
    const params = [restored, AesGcm.xorParams(restored, FILE_NAME_XOR_MASK), AesGcm.xorParams(restored, CHALLENGE_XOR_MASK)]
    expect(restored).toEqual(base)
    for (const param of params) {
        expect(param.iv.byteLength).toBe(12)
        expect(param.tagLength).toBe(128)
    }
    expect(new Set(params.map(param => Buffer.from(param.iv as Uint8Array).toString('hex'))).size).toBe(3)
    expect(restored).toEqual(base)
})
