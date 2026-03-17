export class DerivedKey {
    constructor(public readonly key: CryptoKey, public readonly salt: Uint8Array) {}
}

export default class Pbkdf2 {
    static async deriveKey(password: string, salt = crypto.getRandomValues(new Uint8Array(16))) {
        const iterations = Number.parseInt(process.env.NEXT_PUBLIC_PBKDF2_ITERATIONS ?? '600000')
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
            true,
            ['encrypt', 'decrypt']
        )
        return new DerivedKey(derivedKey, salt)
    }
}
