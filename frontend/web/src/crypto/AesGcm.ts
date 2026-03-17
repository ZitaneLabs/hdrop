// This mask is XORed with the IV to generate the challenge IV.
// This is done to prevent catastrophic failure of AES-GCM through IV reuse.
export const CHALLENGE_XOR_MASK = new Uint8Array([
    0x61, 0xf8, 0x2a, 0x7a, 0xaf, 0x04,
    0x9e, 0x1a, 0xbd, 0xd2, 0x78, 0xfb,
])

// This mask is XORed with the IV to generate the file name IV.
// This is done to prevent catastrophic failure of AES-GCM through IV reuse.
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
        const iv = crypto.getRandomValues(new Uint8Array(12))
        return {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
    }

    /** Restore AES-GCM parameters from an IV. */
    static restoreParams(iv: Uint8Array): AesGcmParams {
        return {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
    }

    /**
     * XOR AES-GCM parameters with a known value.
     * This is done to prevent catastrophic failure of AES-GCM through IV reuse.
     */
    static xorParams(params: AesGcmParams, mask: Uint8Array): AesGcmParams {
        const iv = AesGcm.xorIv(params.iv as Uint8Array, mask)
        return {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
    }

    /** XOR a 12-byte IV with a 12-byte mask. */
    static xorIv(iv: Uint8Array, mask: Uint8Array): Uint8Array {
        console.assert(iv.length === 12, "IV must be 12 bytes long!")
        console.assert(iv.length === 12, "Mask must be 12 bytes long!")
        const result = new Uint8Array(iv.length)
        for (let i = 0; i < iv.length; i++) {
            result[i] = iv[i] ^ mask[i]
        }
        return result
    }

    static async encrypt(data: BufferSource, key: CryptoKey, params: AesGcmParams): Promise<ArrayBuffer> {
        return await crypto.subtle.encrypt(
            params,
            key,
            data
        )
    }

    static async decrypt(data: BufferSource, key: CryptoKey, params: AesGcmParams): Promise<ArrayBuffer> {
        return await crypto.subtle.decrypt(
            params,
            key,
            data
        )
    }
}
