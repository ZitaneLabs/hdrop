import Base64Util from "./Base64Util"

/**
 * Holds derived PBKDF2 key data.
 * Includes PBKDF2 salt and AES-GCM IV.
 */
class DerivedKeyInfo {
    /**
     * 
     * @param {CryptoKey} key 
     * @param {Uint8Array} salt 
     * @param {Uint8Array} iv
     */
    constructor(key, salt, iv) {
        this.key = key
        this.salt = salt
        this.iv = iv
    }
}

/**
 * Holds encrypted file data.
 * 
 * Includes encrypted file contents and file name,
 * as well as derived key info and utilities for
 * converting data to base64.
 */
class EncryptedFileInfo {
    /**
     * 
     * @param {Uint8Array} file_data 
     * @param {Uint8Array} file_name_data 
     * @param {DerivedKeyInfo} derived_key_info 
     */
    constructor(file_data, file_name_data, derived_key_info) {
        this.file_data = file_data
        this.file_name_data = file_name_data
        this.derived_key_info = derived_key_info
    }

    fileDataBase64() {
        return Base64Util.encode(this.file_data)
    }

    fileNameDataBase64() {
        return Base64Util.encode(this.file_name_data)
    }

    ivBase64() {
        return Base64Util.encode(this.derived_key_info.iv)
    }

    saltBase64() {
        return Base64Util.encode(this.derived_key_info.salt)
    }
}

class CryptoUtil {
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
}

export default CryptoUtil
export { DerivedKeyInfo, EncryptedFileInfo }