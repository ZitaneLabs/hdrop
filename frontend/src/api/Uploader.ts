import { AesGcm, CryptoHelper, FILE_NAME_XOR_MASK, Pbkdf2, Sha256 } from "@/crypto"
import { ApiClient, UploadFileData } from "./"
import { CHALLENGE_XOR_MASK } from "@/crypto/AesGcm"

export type UploadPhase = "encrypting" | "uploading" | "done"
export type UploadResult = {
    accessToken: string
    updateToken: string
    password: string
}

export type UploadProgressHandler = (phase: UploadPhase, progress: number) => void
export type UploadCompleteHandler = (result: UploadResult) => void

export default class Uploader {
    static async uploadFile(file: File, onProgressChange: UploadProgressHandler, onUploadComplete: UploadCompleteHandler) {
        onProgressChange('encrypting', 0);

        // Derive a secure random key using PBKDF2
        const passwordString = CryptoHelper.generatePassword()
        const derivedKey = await Pbkdf2.deriveKey(passwordString)

        // Generate encryption params
        const aesBaseParams = AesGcm.generateParams()
        const aesChallengeParams = AesGcm.xorParams(aesBaseParams, CHALLENGE_XOR_MASK)
        const aesFileNameParams = AesGcm.xorParams(aesBaseParams, FILE_NAME_XOR_MASK)

        // Generate a challenge
        const challenge = CryptoHelper.generateChallenge()
        const encryptedChallenge = await AesGcm.encrypt(challenge, derivedKey.key, aesChallengeParams)
        const challengeHash = await Sha256.hash(challenge)

        // Encrypt the file
        const fileBuffer = await file.arrayBuffer()
        const encryptedFile = await AesGcm.encrypt(fileBuffer, derivedKey.key, aesBaseParams)

        // Encrypt the file name
        const fileNameBuffer = new TextEncoder().encode(file.name)
        const encryptedFileName = await AesGcm.encrypt(fileNameBuffer, derivedKey.key, aesFileNameParams)

        // Create a new UploadFileData object
        const data = new UploadFileData(
            aesBaseParams.iv as Uint8Array,
            derivedKey.salt,
            encryptedFile,
            encryptedFileName,
            encryptedChallenge,
            challengeHash,
        )

        // Upload file
        onProgressChange('uploading', 0);
        const resp = await ApiClient.uploadFile(data, progress => {
            onProgressChange('uploading', progress);
        })

        onUploadComplete({
            accessToken: resp.access_token,
            updateToken: resp.update_token,
            password: passwordString,
        })
        onProgressChange('done', 0);
    }
}
