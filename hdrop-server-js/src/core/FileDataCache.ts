export default class FileDataCache {
    cache: Map<string, string> = new Map()

    /**
     * Commit the specified file to the cache.
     */
    commit(accessToken: string, fileData: string) {
        this.cache.set(accessToken, fileData)
    }

    /**
     * Get the specified file from the cache.
     */
    get(accessToken: string): string | null {
        return this.cache.get(accessToken) || null
    }

    /**
     * Evict the specified file from the cache.
     */
    evict(accessToken: string) {
        this.cache.delete(accessToken)
    }
}