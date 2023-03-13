import { Gauge, Histogram } from 'prom-client'

class Metrics {
    cachedFileCount = new Gauge({
        name: 'hdrop_cached_file_count',
        help: 'Number of files stored in the cache',
    })
    cachedFileBytes = new Gauge({
        name: 'hdrop_cached_file_bytes',
        help: 'Number of bytes stored in the cache',
    })
}

export default class FileDataCache {
    cache: Map<string, Buffer> = new Map()
    metrics = new Metrics()

    /**
     * Commit the specified file to the cache.
     */
    commit(accessToken: string, fileData: Buffer) {
        this.cache.set(accessToken, fileData)

        // Update metrics
        this.metrics.cachedFileCount.inc()
        this.metrics.cachedFileBytes.inc(fileData.byteLength)
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

        // Update metrics
        this.metrics.cachedFileCount.dec()
        this.metrics.cachedFileBytes.dec(fileData.byteLength)

        // Evict file from cache
        this.cache.delete(accessToken)
    }
}