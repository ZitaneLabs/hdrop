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
     * Evict the specified file from the cache immediately.
     */
    evictImmediately(accessToken: string) {
        this.cache.delete(accessToken)
    }

    /**
     * Evict the specified file from the cache after the specified timeout.
     * 
     * @param timeout Timeout in milliseconds
     */
    evictAfter(accessToken: string, timeout: number) {
        setTimeout(() => this.evictImmediately(accessToken), timeout)
    }
}