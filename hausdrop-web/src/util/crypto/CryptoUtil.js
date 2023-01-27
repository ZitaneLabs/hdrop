import { DerivedKeyInfo, EncryptedFileInfo, DecryptedFileInfo } from ".."

/**
 * A utility class for encrypting and decrypting data.
 */
export default class CryptoUtil {
    /**
     * Derives a PBKDF2 key for AES-GCM encryption.
     * 
     * @param {string} password
     * @returns {Promise<DerivedKeyInfo>}
    */
    static async deriveKeyFromPassword(password) {
        const textEncoder = new TextEncoder()
        const passwordBuffer = textEncoder.encode(password)
        const importedKey = await window.crypto.subtle.importKey('raw', passwordBuffer, 'PBKDF2', false, ['deriveKey'])
        const salt = window.crypto.getRandomValues(new Uint8Array(16))
        const derivedKey = await window.crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt,
                iterations: 100000,
                hash: 'SHA-256',
            },
            importedKey,
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        )
        const iv = window.crypto.getRandomValues(new Uint8Array(12))
        return new DerivedKeyInfo(derivedKey, salt, iv)
    }

    /**
     *  Recovers a PBKDF2 key for AES-GCM encryption.
     * 
     * @param {string} password
     * @param {Uint8Array} salt
     * @param {Uint8Array} iv
     * @returns {Promise<DerivedKeyInfo>}
     */
    static async recoverKeyFromPassword(password, salt, iv) {
        const textEncoder = new TextEncoder()
        const passwordBuffer = textEncoder.encode(password)
        const importedKey = await window.crypto.subtle.importKey('raw', passwordBuffer, 'PBKDF2', false, ['deriveKey'])
        const derivedKey = await window.crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt,
                iterations: 100000,
                hash: 'SHA-256',
            },
            importedKey,
            { name: 'AES-GCM', length: 256 },
            true,
            ['encrypt', 'decrypt']
        )
        return new DerivedKeyInfo(derivedKey, salt, iv)
    }

    /**
     * Encrypts a file using AES-GCM.
     * 
     * @param {Uint8Array} data 
     * @param {DerivedKeyInfo} derivedKeyInfo 
     * @returns {Promise<ArrayBuffer>}
     */
    static async encryptData(data, derivedKeyInfo) {
        return await window.crypto.subtle.encrypt(
            {
                name: 'AES-GCM',
                iv: derivedKeyInfo.iv,
                additionalData: new ArrayBuffer(0),
                tagLength: 128,
            },
            derivedKeyInfo.key,
            data
        )
    }

    /**
     * Decrypts a file using AES-GCM.
     * 
     * @param {Uint8Array} data 
     * @param {DerivedKeyInfo} derivedKeyInfo 
     * @returns {Promise<ArrayBuffer>}
     */
    static async decryptData(data, derivedKeyInfo) {
        return await window.crypto.subtle.decrypt(
            {
                name: 'AES-GCM',
                iv: derivedKeyInfo.iv,
                additionalData: new ArrayBuffer(0),
                tagLength: 128,
            },
            derivedKeyInfo.key,
            data
        )
    }

    /**
     * 
     * @param {string} str 
     * @param {DerivedKeyInfo} derivedKeyInfo  
     * @returns {Promise<ArrayBuffer>}
     */
    static async encryptString(str, derivedKeyInfo) {
        const textEncoder = new TextEncoder()
        const data = textEncoder.encode(str)
        return await CryptoUtil.encryptData(data, derivedKeyInfo)
    }

    /**
     * 
     * @param {Uint8Array} data
     * @param {DerivedKeyInfo} derivedKeyInfo  
     * @returns {Promise<ArrayBuffer>}
     */
    static async decryptString(data, derivedKeyInfo) {
        const textDecoder = new TextDecoder()
        const decryptedData = await CryptoUtil.decryptData(data, derivedKeyInfo)
        return textDecoder.decode(decryptedData)
    }

    /**
     * @param {Uint8Array} fileData
     * @param {string} fileName
     * @param {DerivedKeyInfo} derivedKeyInfo
     * @returns {Promise<EncryptedFileInfo>}
     */
    static async encryptFile(fileData, fileName, derivedKeyInfo) {
        const fileDataEncrypted = new Uint8Array(await CryptoUtil.encryptData(fileData, derivedKeyInfo))
        const fileNameEncrypted = new Uint8Array(await CryptoUtil.encryptString(fileName, derivedKeyInfo))
        return new EncryptedFileInfo(fileDataEncrypted, fileNameEncrypted, derivedKeyInfo)
    }

    /**
     * @param {Uint8Array} encryptedFileData
     * @param {string} fileName
     * @param {DerivedKeyInfo} derivedKeyInfo
     * @returns {Promise<DecryptedFileInfo>}
     */
    static async decryptFile(encryptedFileData, fileName, derivedKeyInfo) {
        const fileData = new Uint8Array(await CryptoUtil.decryptData(encryptedFileData, derivedKeyInfo))
        return new DecryptedFileInfo(fileData, fileName)
    }
}