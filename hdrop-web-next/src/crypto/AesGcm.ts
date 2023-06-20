export class AesBundle {
    constructor(public readonly iv: Uint8Array, public readonly data: ArrayBuffer) {}
}

export default class AesGcm {
    static async encrypt(data: BufferSource, key: CryptoKey): Promise<AesBundle> {
        const iv = crypto.getRandomValues(new Uint8Array(12))
        let params: AesGcmParams = {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
        const encryptedData = await crypto.subtle.encrypt(
            params,
            key,
            data
        )
        return new AesBundle(iv, encryptedData)
    }

    static async decrypt(data: BufferSource, key: CryptoKey, iv: Uint8Array): Promise<ArrayBuffer> {
        let params: AesGcmParams = {
            name: 'AES-GCM',
            iv,
            tagLength: 128,
        }
        return await crypto.subtle.decrypt(
            params,
            key,
            data
        )
    }
}
