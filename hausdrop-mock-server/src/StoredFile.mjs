import { v4 as uuidv4 } from 'uuid'

import R2Provider from './R2Provider.mjs'
import TokenGenerator from './TokenGenerator.mjs'

/**
 * Number of milliseconds to wait before persisting a file.
 * This will be reset every time the file is accessed.
 * 
 * TODO: This should be `const`.
 * 
 * @type {number}
 */
let PERSIST_TIMEOUT = 1000 * 60 * 5 // 5 minutes

// Check if debug mode
if (process.env.DEBUG) {
    PERSIST_TIMEOUT = 1000 * 30 // 30 seconds
}

export default class StoredFile {
    /**
     * Cache statistics.
     * 
     * @type {{
     * hits: number,
     * misses: number,
     * }}
     */
    static cacheStatistics = {
        hits: 0,
        misses: 0,
    }

    /** @type {R2Provider} */
    provider

    /** @type {string} */
    uuid

    /** @type {string} */
    accessToken

    /** @type {string} */
    updateToken

    /** @type {string} */
    _fileData

    /** @type {string} */
    fileNameData

    /** @type {string} */
    fileNameHash

    /** @type {string} */
    salt

    /** @type {string} */
    iv

    /** @type {boolean} */
    persisted

    /** @type {number} */
    lastTimeoutId

    /**
     * Construct a new `StoredFile` instance.
     * 
     * @param {R2Provider} provider
     * @param {{
     * fileData: string,
     * fileNameData: string,
     * fileNameHash: string,
     * salt: string,
     * iv: string,
     * }} data
     */
    constructor(provider, { fileData, fileNameData, fileNameHash, salt, iv }) {
        // Store provider
        this.provider = provider

        // Generate UUID
        this.uuid = uuidv4()

        // Generate tokens
        this.accessToken = TokenGenerator.generateAccessToken()
        this.updateToken = TokenGenerator.generateUpdateToken()

        // Store data
        this._fileData = fileData
        this.fileNameData = fileNameData
        this.fileNameHash = fileNameHash
        this.salt = salt
        this.iv = iv

        // Mark as not persisted
        this.persisted = false

        // No timeout scheduled
        this.lastTimeoutId = null

        // Schedule for persistence
        this.smartPersist()
    }

    /**
     * Persist file contents to remote storage.
     */
    async forcePersist() {
        // Guard against persisting twice
        if (this.persisted) {

            // Check if fileData is still in memory
            if (this._fileData !== null) {
                console.log(`[StoredFile/watchdog] ${this.accessToken} has gone cold`)

                // If we hit this case, the file is cold
                // and we can safely GC it.
                this._fileData = null
            }

            // Return early
            return
        }

        // Persist to remote storage
        console.log(`[StoredFile/persist] Persisting ${this.accessToken} to remote`)
        await this.provider.uploadFile(this.uuid, this._fileData)
        console.log(`[StoredFile/persist] Successfully persisted ${this.accessToken}`)

        // Mark fileData as GC-able
        this._fileData = null

        // Mark as persisted
        this.persisted = true
    }

    /**
     * Persist file contents to remote storage after a delay.
     */
    smartPersist() {
        // Clear previous timeout
        if (this.lastTimeoutId !== null) {
            clearTimeout(this.lastTimeoutId)
        }

        // Schedule for persistence
        this.lastTimeoutId = setTimeout(() => this.forcePersist(), PERSIST_TIMEOUT)
    }

    /**
     * Delete file contents from remote storage.
     */
    async cleanup() {
        // Mark as not persisted
        this.persisted = false

        // Delete from remote storage
        await this.provider.deleteFile(this.uuid)

        // Cleanup remaining memory
        this._fileData = null
    }

    /**
     * Retrieve file contents from remote storage.
     */
    async fileData() {
        // Persisted and cold
        if (this._fileData === null && this.persisted) {
            // Update cache statistics
            StoredFile.cacheStatistics.misses += 1

            // Retrieve from remote storage
            console.log(`[StoredFile/cache] Retrieving ${this.accessToken} from remote`)
            const data = await this.provider.downloadFile(this.uuid)

            // Store in memory
            this._fileData = data

            // Schedule for persistence
            this.smartPersist()

            // Return data
            return data
        }

        // Persisted and still hot or not persisted
        else {
            // Update cache statistics
            StoredFile.cacheStatistics.hits += 1

            // Schedule for persistence
            this.smartPersist()

            // Serve from memory
            return this._fileData
        }
    }

    static printCacheStatistics() {
        const { hits, misses } = this.cacheStatistics
        const hitRatio = (hits / (hits + misses)).toFixed(2)
        console.log(`[StoredFile/cache] { hit: ${hits}; miss: ${misses}; ratio: ${hitRatio} }`)
    }
}