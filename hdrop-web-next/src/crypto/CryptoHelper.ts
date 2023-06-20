import { AesGcm, DataBundle, Pbkdf2 } from './'

export default class CryptoHelper {
    static async encrypt(data: BufferSource, password: string): Promise<DataBundle> {
        const derived_key = await Pbkdf2.deriveKey(password)
        const aes_bundle = await AesGcm.encrypt(data, derived_key.key)
        return new DataBundle(derived_key, aes_bundle)
    }

    static async decrypt(data: DataBundle): Promise<ArrayBuffer> {
        return await AesGcm.decrypt(data.aes_bundle.data, data.derived_key.key, data.aes_bundle.iv)
    }
}
