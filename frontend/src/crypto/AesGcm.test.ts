/** @jest-environment node */
import AesGcm, { FILE_AAD, FILE_NAME_AAD, CHALLENGE_AAD, FILE_NAME_XOR_MASK, CHALLENGE_XOR_MASK } from './AesGcm'

const domains = [FILE_AAD, FILE_NAME_AAD, CHALLENGE_AAD]
let key: CryptoKey

afterEach(() => jest.restoreAllMocks())

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
    const random = jest.spyOn(crypto, 'getRandomValues')
    const base = AesGcm.generateParams()
    expect(random).toHaveBeenCalledWith(base.iv)
    expect(CHALLENGE_XOR_MASK.some(byte => byte !== 0)).toBe(true)
    expect(FILE_NAME_XOR_MASK.some(byte => byte !== 0)).toBe(true)
    expect(CHALLENGE_XOR_MASK).not.toEqual(FILE_NAME_XOR_MASK)
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

test('XORs all 12 bytes without mutating either input', () => {
    const iv = new Uint8Array(12).fill(0xff)
    const mask = new Uint8Array(12).fill(0xaa)
    expect(AesGcm.xorIv(iv, mask)).toEqual(new Uint8Array(12).fill(0x55))
    expect(iv).toEqual(new Uint8Array(12).fill(0xff))
    expect(mask).toEqual(new Uint8Array(12).fill(0xaa))
})

test.each([0, 1, 11, 13, 16])('rejects %i-byte IVs before WebCrypto', async length => {
    const iv = new Uint8Array(length)
    expect(() => AesGcm.restoreParams(iv)).toThrow('IV must be 12 bytes')
    expect(() => AesGcm.xorIv(iv, CHALLENGE_XOR_MASK)).toThrow('IV must be 12 bytes')
    const params = { ...AesGcm.generateParams(), iv }
    for (const operation of ['encrypt', 'decrypt'] as const) {
        const native = jest.spyOn(crypto.subtle, operation)
        await expect(AesGcm[operation](iv, key, params, FILE_AAD)).rejects.toThrow('IV must be 12 bytes')
        expect(native).not.toHaveBeenCalled()
    }
})

test.each([new Uint8Array(0), new Uint8Array(11), new Uint8Array(13), new Uint8Array(12), Array(12).fill(1), null])(
    'rejects invalid mask %p', mask => {
        expect(() => AesGcm.xorParams(AesGcm.generateParams(), mask as Uint8Array)).toThrow('Mask must be 12 bytes long and non-zero')
    },
)

test.each(['ciphertext', 'tag', 'IV', 'key'])('rejects tampered %s', async target => {
    const params = AesGcm.generateParams()
    const ciphertext = new Uint8Array(await AesGcm.encrypt(new Uint8Array([1, 2, 3]), key, params, FILE_AAD))
    let decryptKey = key
    if (target === 'ciphertext') ciphertext[0] ^= 1
    if (target === 'tag') ciphertext[ciphertext.length - 1] ^= 1
    if (target === 'IV') (params.iv as Uint8Array)[0] ^= 1
    if (target === 'key') decryptKey = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt'])
    await expect(AesGcm.decrypt(ciphertext, decryptKey, params, FILE_AAD)).rejects.toMatchObject({ name: 'OperationError' })
})
