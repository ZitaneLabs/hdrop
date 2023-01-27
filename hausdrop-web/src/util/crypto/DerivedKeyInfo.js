/**
 * Holds derived PBKDF2 key data.
 * Includes PBKDF2 salt and AES-GCM IV.
 */
export default class DerivedKeyInfo {
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