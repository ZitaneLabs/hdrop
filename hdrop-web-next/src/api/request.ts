import { AesGcm, Base64, CryptoHelper, Pbkdf2, Sha256 } from "@/crypto"

export class UploadFileData {
    constructor(
        public readonly iv: Uint8Array,
        public readonly salt: Uint8Array,
        public readonly fileData: ArrayBuffer,
        public readonly fileNameData: ArrayBuffer,
        public readonly challengeData: ArrayBuffer,
        public readonly challengeHash: string,
    ) {}

    toMultipart(): FormData {
        const iv = Base64.encode(this.iv)
        const salt = Base64.encode(this.salt)
        const fileData = new Blob([this.fileData])
        const fileNameData = Base64.encode(new Uint8Array(this.fileNameData))
        const rawChallengeData = new Uint8Array(this.challengeData)
        const challengeData = Base64.encode(rawChallengeData)

        const formData = new FormData()
        formData.set('iv', iv)
        formData.set('salt', salt)
        formData.set('file_data', fileData)
        formData.set('file_name_data', fileNameData)
        formData.set('challenge_data', challengeData)
        formData.set('challenge_hash', this.challengeHash)
        return formData
    }
}
