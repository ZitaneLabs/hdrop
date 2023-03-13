export default class FileDataCache {
    cache: Map<string, Buffer> = new Map()

    /**
     * Commit the specified file to the cache.
     */
    commit(accessToken: string, fileData: Buffer) {
        this.cache.set(accessToken, fileData)
    }

    /**
     * Get the specified file from the cache.
     */
    get(accessToken: string): Buffer | null {
        return this.cache.get(accessToken) || null
    }

    /**
     * Evict the specified file from the cache.
     */
    evict(accessToken: string) {
        if (!this.cache.has(accessToken)) return;
        const fileData = this.cache.get(accessToken)!


        // Evict file from cache
        this.cache.delete(accessToken)
    }
}