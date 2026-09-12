// Stable UTF-8 domains authenticate each ciphertext's protocol role.
export const FILE_AAD = new TextEncoder().encode('hdrop/v1/file')
export const FILE_NAME_AAD = new TextEncoder().encode('hdrop/v1/filename')
export const CHALLENGE_AAD = new TextEncoder().encode('hdrop/v1/challenge')

// Public, distinct, non-zero masks separate IVs within one upload.
// Each upload must derive a fresh key; these masks do not ensure global uniqueness.
export const CHALLENGE_XOR_MASK = new Uint8Array([
    0x61, 0xf8, 0x2a, 0x7a, 0xaf, 0x04,
    0x9e, 0x1a, 0xbd, 0xd2, 0x78, 0xfb,
])

// XORed with the base IV to generate the file name IV.
export const FILE_NAME_XOR_MASK = new Uint8Array([
    0x92, 0x6a, 0x41, 0xdf, 0x67, 0xa0,
    0x3f, 0x8a, 0x0a, 0x7b, 0xd7, 0x9c,
])

export class AesBundle {
    constructor(public readonly iv: Uint8Array, public readonly data: ArrayBuffer) {}
}

export default class AesGcm {
    /** Generate AES-GCM parameters. */
    static generateParams(): AesGcmParams {
        return AesGcm.restoreParams(crypto.getRandomValues(new Uint8Array(12)))
    }

    /** Restore AES-GCM parameters from an IV. */
    static restoreParams(iv: Uint8Array): AesGcmParams {
        if (!(iv instanceof Uint8Array) || iv.length !== 12) throw new RangeError('IV must be 12 bytes long')
        return {
            name: 'AES-GCM',
            iv: iv as BufferSource,
            tagLength: 128,
        }
    }

    /** Derive another IV within the same encryption context. */
    static xorParams(params: AesGcmParams, mask: Uint8Array): AesGcmParams {
        const iv = AesGcm.xorIv(params.iv as Uint8Array, mask)
        return AesGcm.restoreParams(iv)
    }

    /** XOR a 12-byte IV with a 12-byte mask. */
    static xorIv(iv: Uint8Array, mask: Uint8Array): Uint8Array {
        if (!(iv instanceof Uint8Array) || iv.length !== 12) throw new RangeError('IV must be 12 bytes long')
        if (!(mask instanceof Uint8Array) || mask.length !== 12 || !mask.some(byte => byte !== 0)) {
            throw new RangeError('Mask must be 12 bytes long and non-zero')
        }
        const result = new Uint8Array(iv.length)
        for (let i = 0; i < iv.length; i++) {
            result[i] = iv[i] ^ mask[i]
        }
        return result
    }

    static async encrypt(data: ArrayBuffer | Uint8Array, key: CryptoKey, params: AesGcmParams, additionalData: BufferSource): Promise<ArrayBuffer> {
        if (params.iv.byteLength !== 12) throw new RangeError('IV must be 12 bytes long')
        return await crypto.subtle.encrypt(
            { ...params, additionalData },
            key,
            data as BufferSource
        )
    }

    static async decrypt(data: ArrayBuffer | Uint8Array, key: CryptoKey, params: AesGcmParams, additionalData: BufferSource): Promise<ArrayBuffer> {
        if (params.iv.byteLength !== 12) throw new RangeError('IV must be 12 bytes long')
        return await crypto.subtle.decrypt(
            { ...params, additionalData },
            key,
            data as BufferSource
        )
    }
}
