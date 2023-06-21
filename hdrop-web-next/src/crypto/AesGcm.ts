export class AesBundle {
    constructor(public readonly iv: Uint8Array, public readonly data: ArrayBuffer) {}
}

export default class AesGcm {
    static generateParams(): AesGcmParams {
        const iv = crypto.getRandomValues(new Uint8Array(12))
        return {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
    }

    static restoreParams(iv: Uint8Array): AesGcmParams {
        return {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
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
