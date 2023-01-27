import {CryptoUtil, Base64Util} from '..'

/**
 * Holds encrypted file data.
 * 
 * Includes encrypted file contents and file name,
 * as well as derived key info and utilities for
 * converting data to base64.
 */
export default class EncryptedFileInfo {
    /**
     * 
     * @param {Uint8Array} file_data 
     * @param {Uint8Array} file_name_data 
     * @param {string} file_name_hash
     * @param {DerivedKeyInfo} derived_key_info 
     */
    constructor(file_data, file_name_data, file_name_hash, derived_key_info) {
        this.file_data = file_data
        this.file_name_data = file_name_data
        this.file_name_hash = file_name_hash
        this.derived_key_info = derived_key_info
    }

    /**
     * Construct an EncryptedFileInfo from previously encrypted file data.
     * 
     * @param {{
     * file_data: string,
     * file_name_data: string,
     * iv: string,
     * salt: string,
     * }} data Base64 Data
     * @param {string} password Password
     */
    static async fromEncryptedFileData(data, password) {
        console.log(data)
        const salt = Base64Util.decode(data.salt)
        const iv = Base64Util.decode(data.iv)
        const derivedKeyInfo = await CryptoUtil.recoverKeyFromPassword(password, salt, iv)
        console.log(derivedKeyInfo)
        return new EncryptedFileInfo(
            Base64Util.decode(data.file_data),
            Base64Util.decode(data.file_name_data),
            null,
            derivedKeyInfo,
        )
    }

    /**
     * Returns the file data as a base64 string.
     * 
     * @returns {string}
     */
    fileDataBase64() {
        return Base64Util.encode(this.file_data)
    }

    /**
     * Returns the file name data as a base64 string.
     * 
     * @returns {string}
     */
    fileNameDataBase64() {
        return Base64Util.encode(this.file_name_data)
    }

    /**
     * Returns the IV as a base64 string.
     * 
     * @returns {string}
     */
    ivBase64() {
        return Base64Util.encode(this.derived_key_info.iv)
    }

    /**
     * Returns the salt as a base64 string.
     * 
     * @returns {string}
     */
    saltBase64() {
        return Base64Util.encode(this.derived_key_info.salt)
    }

    /**
     * Returns the file name hash.
     * Used as a challenge for the user to decrypt the file.
     * 
     * @returns {string}
     */
    challenge() {
        return this.file_name_hash
    }

    /**
     * Decrypts the file data and returns a DecryptedFileInfo.
     * 
     * @returns {Promise<DecryptedFileInfo>}
     */
    async decrypt() {
        const fileName = await CryptoUtil.decryptString(this.file_name_data, this.derived_key_info)
        return await CryptoUtil.decryptFile(this.file_data, fileName, this.derived_key_info)
    }
}