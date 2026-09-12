import { cryptoInteger } from './config'

export class DerivedKey {
    constructor(public readonly key: CryptoKey, public readonly salt: Uint8Array) {}
}

export default class Pbkdf2 {
    static async deriveKey(password: string, salt?: Uint8Array<ArrayBuffer>) {
        const iterations = cryptoInteger(process.env.NEXT_PUBLIC_PBKDF2_ITERATIONS, 'PBKDF2 iterations', 600000, 0xffffffff)
        salt ??= crypto.getRandomValues(new Uint8Array(16))
        const textEncoder = new TextEncoder()
        const passwordBuffer = textEncoder.encode(password)
        const importedKey = await crypto.subtle.importKey('raw', passwordBuffer, 'PBKDF2', false, ['deriveKey'])
        const params: Pbkdf2Params = {
            name: 'PBKDF2',
            salt,
            iterations,
            hash: 'SHA-256',
        }
        const derivedKey = await crypto.subtle.deriveKey(
            params,
            importedKey,
            { name: 'AES-GCM', length: 256 },
            false,
            ['encrypt', 'decrypt']
        )
        return new DerivedKey(derivedKey, salt)
    }
}
