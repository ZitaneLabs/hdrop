export default class FileDataCache {
    /**
     * @type {Map<string, string>}
     */
    cache = new Map()

    /**
     * Commit the specified file to the cache.
     * 
     * @param {string} accessToken
     * @param {string} fileData
     */
    commit(accessToken, fileData) {
        this.cache.set(accessToken, fileData)
    }

    /**
     * Get the specified file from the cache.
     * 
     * @param {string} accessToken
     * @returns {string | null}
     */
    get(accessToken) {
        return this.cache.get(accessToken) || null
    }

    /**
     * Evict the specified file from the cache.
     * 
     * @param {string} accessToken
     */
    evict(accessToken) {
        this.cache.delete(accessToken)
    }
}