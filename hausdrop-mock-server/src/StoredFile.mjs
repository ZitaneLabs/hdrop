import { v4 as uuidv4 } from 'uuid'

import S3Provider from './S3Provider.mjs'
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

export class ExportFileData {
    /**
     * @type {'local' | 'remote'}
     */
    type

    /**
     * @type {string | null}
     */
    _fileData

    /**
     * @type {string | null}
     */
    _fileUrl

    /**
     * @param {'local' | 'remote'} type
     * @param {string | null} fileData
     * @param {string | null} fileUrl
     */
    constructor(type, fileData, fileUrl) {
        this.type = type
        this._fileData = fileData
        this._fileUrl = fileUrl
    }

    /**
     * Get base64 file data.
     * 
     * @returns {string | null}
     */
    fileData() {
        return this._fileData
    }

    /**
     * Get remote file URL.
     * 
     * @returns {string | null}
     */
    fileUrl() {
        return this._fileUrl
    }

    /**
     * @param {string} fileData
     */
    static fromLocal(fileData) {
        return new ExportFileData('local', fileData, null)
    }

    /**
     * @param {string} fileUrl
     */
    static fromRemote(fileUrl) {
        return new ExportFileData('remote', null, fileUrl)
    }
}

export default class StoredFile {
    /**
     * Cache statistics.
     * 
     * @type {{
     * memory: number,
     * remote: number,
     * }}
     */
    static cacheStatistics = {
        memory: 0,
        remote: 0,
    }

    /** @type {S3Provider} */
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

    /** @type {string} */
    bucketUrl

    /** @type {number} */
    lastTimeoutId

    /**
     * Construct a new `StoredFile` instance.
     * 
     * @param {S3Provider} provider
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
        this.bucketUrl = null

        // Schedule persist
        setImmediate(() => this.persist())
    }

    /**
     * Persist file contents to remote storage.
     */
    async persist() {
        // Guard against persisting twice
        if (this.isPersisted()) {

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
        console.log(`[StoredFile/persist ${this.accessToken}] Persisting to remote`)
        await this.provider.uploadFile(this.uuid, this._fileData)
        console.log(`[StoredFile/persist ${this.accessToken}] Successfully persisted`)

        // Mark fileData as GC-able
        this._fileData = null

        // Mark as persisted
        this.bucketUrl = this.provider.buildUrl(this.uuid)
    }

    /**
     * Check if file contents are persisted to remote storage.
     * 
     * @returns {boolean}
     */
    isPersisted() {
        return this.bucketUrl !== null
    }

    /**
     * Delete file contents from remote storage.
     */
    async cleanup() {
        // Mark as not persisted
        this.bucketUrl = null

        // Delete from remote storage
        await this.provider.deleteFile(this.uuid)

        // Cleanup remaining memory
        this._fileData = null
    }

    /**
     * Get file data.
     * 
     * @returns {Promise<ExportFileData>}
     */
    async exportFileData() {
        // Persisted
        if (this.isPersisted()) {
            // Update cache statistics
            StoredFile.cacheStatistics.remote += 1

            // Serve from remote storage
            return ExportFileData.fromRemote(this.bucketUrl)
        }

        // In-Memory
        else {
            // Update cache statistics
            StoredFile.cacheStatistics.memory += 1

            // Schedule persist
            setImmediate(() => this.persist())

            // Serve from memory
            return ExportFileData.fromLocal(this._fileData)
        }
    }

    static printCacheStatistics() {
        const { memory, remote } = this.cacheStatistics
        console.log(`[StoredFile/cache] { memory: ${memory}; remote: ${remote}; }`)
    }
}