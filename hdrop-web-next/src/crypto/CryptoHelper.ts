import { AesGcm, Base64 } from './'

export default class CryptoHelper {
    static generatePassword(): string {
        const length = Number.parseInt(process.env.NEXT_PUBLIC_PASSWORD_BYTES ?? '32')
        const bytes = crypto.getRandomValues(new Uint8Array(length))
        return Base64.encode(bytes)
    }

    static generateChallenge(): Uint8Array {
        const length = Number.parseInt(process.env.NEXT_PUBLIC_CHALLENGE_BYTES ?? '32')
        return crypto.getRandomValues(new Uint8Array(length))
    }
}
