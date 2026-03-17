export default class Sha256 {
    static async hash(data: Uint8Array<ArrayBufferLike>): Promise<string> {
        const hashData = await window.crypto.subtle.digest('SHA-256', data as BufferSource)
        const hashArray = Array.from(new Uint8Array(hashData))
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
    }
}
