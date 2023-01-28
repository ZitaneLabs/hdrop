import StoredFile from "./StoredFile.mjs"

export default class FileStorage {
    /**
     * Stored files
     * 
     * @type {Record<string, StoredFile>}
     */
    files = {}

    /**
     * Store a file
     * 
     * @param {StoredFile} file
     */
    storeFile(file) {
        this.files[file.accessToken] = file
    }

    /**
     * Retrieve a file
     * 
     * @param {string} accessToken
     * @returns {StoredFile | null}
     */
    retrieveFile(accessToken) {
        return this.files[accessToken] || null
    }

    /**
     * Delete a file
     * 
     * @param {string} accessToken
     * @param {StorageProvider} provider
     */
    async deleteFile(accessToken, provider) {
        const file = this.retrieveFile(accessToken)

        if (file !== null) {
            // Delete from remote storage
            await file.cleanup(provider)

            // Delete from memory
            delete this.files[accessToken]
        }
    }
}