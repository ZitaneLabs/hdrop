/** @jest-environment node */
import { pbkdf2Sync } from 'node:crypto'
import { AesGcm, Base64, CryptoHelper, FILE_AAD, Pbkdf2 } from './'

beforeEach(() => {
    jest.replaceProperty(process, 'env', { ...process.env })
    delete process.env.NEXT_PUBLIC_PBKDF2_ITERATIONS
    delete process.env.NEXT_PUBLIC_PASSWORD_BYTES
    delete process.env.NEXT_PUBLIC_CHALLENGE_BYTES
})
afterEach(() => jest.restoreAllMocks())

test.each([undefined, '600001'])('derives a non-extractable AES-256-GCM key using PBKDF2-SHA256 (%s iterations)', async value => {
    if (value !== undefined) process.env.NEXT_PUBLIC_PBKDF2_ITERATIONS = value
    const salt = new Uint8Array(16).fill(7)
    const { key, salt: returnedSalt } = await Pbkdf2.deriveKey('test password', salt)
    expect(returnedSalt).toEqual(salt)
    expect(key.algorithm).toEqual({ name: 'AES-GCM', length: 256 })
    expect(key.extractable).toBe(false)
    expect(key.usages).toEqual(['encrypt', 'decrypt'])
    await expect(crypto.subtle.exportKey('raw', key)).rejects.toThrow('key is not extractable')

    // An independent implementation verifies hash, iteration count, salt and key size.
    const expected = pbkdf2Sync('test password', salt, Number(value ?? 600000), 32, 'sha256')
    const referenceKey = await crypto.subtle.importKey('raw', expected, 'AES-GCM', false, ['decrypt'])
    const params = AesGcm.generateParams()
    const plaintext = new Uint8Array([1, 2, 3])
    const ciphertext = await AesGcm.encrypt(plaintext, key, params, FILE_AAD)
    expect(new Uint8Array(await AesGcm.decrypt(ciphertext, referenceKey, params, FILE_AAD))).toEqual(plaintext)
})

describe.each([
    ['NEXT_PUBLIC_PASSWORD_BYTES', () => Base64.decode(CryptoHelper.generatePassword())],
    ['NEXT_PUBLIC_CHALLENGE_BYTES', () => CryptoHelper.generateChallenge()],
] as const)('%s', (name, generate) => {
    test.each([undefined, '48', '65536'])('generates the configured random bytes (%s)', value => {
        if (value !== undefined) process.env[name] = value
        const random = jest.spyOn(crypto, 'getRandomValues')
        const bytes = generate()
        expect(bytes).toHaveLength(Number(value ?? 32))
        expect(random).toHaveBeenCalledTimes(1)
        expect(bytes).toEqual(random.mock.calls[0][0])
    })
})

describe.each([
    ['NEXT_PUBLIC_PBKDF2_ITERATIONS', 600000, 0xffffffff, () => Pbkdf2.deriveKey('password')],
    ['NEXT_PUBLIC_PASSWORD_BYTES', 32, 65536, () => CryptoHelper.generatePassword()],
    ['NEXT_PUBLIC_CHALLENGE_BYTES', 32, 65536, () => CryptoHelper.generateChallenge()],
] as const)('%s validation', (name, min, max, run) => {
    test.each(['', ' ', 'NaN', 'Infinity', 'abc', '0', '-1', '32.5', '600000junk', '6e5', '0x100000', String(min - 1), String(max + 1)])(
        'rejects %j before using crypto', async value => {
            process.env[name] = value
            const random = jest.spyOn(crypto, 'getRandomValues')
            const imported = jest.spyOn(crypto.subtle, 'importKey')
            await expect(Promise.resolve().then(async () => { await run() })).rejects.toThrow(RangeError)
            expect(random).not.toHaveBeenCalled()
            expect(imported).not.toHaveBeenCalled()
        },
    )
})
