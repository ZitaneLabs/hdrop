import { Base64 } from './'
import { cryptoInteger } from './config'

export default class CryptoHelper {
    static generatePassword(): string {
        const length = cryptoInteger(process.env.NEXT_PUBLIC_PASSWORD_BYTES, 'Password bytes', 32, 65536)
        const bytes = crypto.getRandomValues(new Uint8Array(length))
        return Base64.encode(bytes)
    }

    static generateChallenge(): Uint8Array {
        const length = cryptoInteger(process.env.NEXT_PUBLIC_CHALLENGE_BYTES, 'Challenge bytes', 32, 65536)
        return crypto.getRandomValues(new Uint8Array(length))
    }
}
