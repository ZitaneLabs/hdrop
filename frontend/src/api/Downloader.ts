import { AesGcm, Base64, CHALLENGE_XOR_MASK, FILE_NAME_XOR_MASK, Pbkdf2, Sha256 } from "@/crypto"
import { ApiClient } from './'

export type DownloadPhase = "validating" | "downloading" | "decrypting" | "done"
export type DownloadResult = {
    data: ArrayBuffer
}

export type DownloadProgressHandler = (phase: DownloadPhase, progress: number) => void
export type DownloadCompleteHandler = (result: DownloadResult) => void
export type DownloadFileNameHandler = (fileName: string) => void
export type DownloadFileParams = {
    accessToken: string
    password: string
    onProgressChange: DownloadProgressHandler
    onFileNameObtained: DownloadFileNameHandler
    onDownloadComplete: DownloadCompleteHandler
}

export default class Downloader {
    static async downloadFile({ accessToken, password, onProgressChange, onFileNameObtained, onDownloadComplete }: DownloadFileParams) {
        onProgressChange('validating', 0);

        // Solve challenge
        const { challenge, salt, iv } = await ApiClient.getChallenge(accessToken)
        const challengeBytes = Base64.decode(challenge)
        const saltBytes = Base64.decode(salt)
        const derivedKey = await Pbkdf2.deriveKey(password, saltBytes)
        const ivBytes = Base64.decode(iv)
        const aesBaseParams = AesGcm.restoreParams(ivBytes)
        const aesChallengeParams = AesGcm.xorParams(aesBaseParams, CHALLENGE_XOR_MASK)
        const aesFileNameParams = AesGcm.xorParams(aesBaseParams, FILE_NAME_XOR_MASK)
        const decryptedChallenge = await AesGcm.decrypt(challengeBytes, derivedKey.key, aesChallengeParams)
        const challengeHash = await Sha256.hash(new Uint8Array(decryptedChallenge))

        // Submit challenge
        const { file_name_data } = await ApiClient.submitChallenge(accessToken, challengeHash)

        // Decrypt file name
        const fileNameBytes = Base64.decode(file_name_data)
        const decryptedFileName = await AesGcm.decrypt(fileNameBytes, derivedKey.key, aesFileNameParams)
        const fileName = new TextDecoder().decode(new Uint8Array(decryptedFileName))
        onFileNameObtained(fileName)

        // Download file
        onProgressChange('downloading', 0);
        const fileBytes = await ApiClient.downloadFile(accessToken, challengeHash, progress => {
            onProgressChange('downloading', progress);
        })

        // Decrypt file
        onProgressChange('decrypting', 1);
        const decryptedFile = await AesGcm.decrypt(fileBytes, derivedKey.key, aesBaseParams)

        onDownloadComplete({ data: decryptedFile })
        onProgressChange('done', 1);
    }
}
