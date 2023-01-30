import Prisma from '@prisma/client'

import StoredFile from "./StoredFile.mjs"
import DatabaseClient from "./DatabaseClient.mjs"
import S3Provider from "./providers/S3Provider.mjs"
import FileDataCache from "./FileDataCache.mjs"

/**
 * File storage coordinator.
 * 
 * Handles database, remote storage and caching.
 */
export default class FileStorage {
    /** @type {DatabaseClient} */
    dbClient

    /** @type {S3Provider} */
    storageProvider

    /** @type {FileDataCache} */
    cache = new FileDataCache()

    /**
     * @param {DatabaseClient} dbClient
     * @param {S3Provider} storageProvider
     */
    constructor(dbClient, storageProvider) {
        this.dbClient = dbClient
        this.storageProvider = storageProvider
    }

    /**
     * Store a file
     * 
     * @param {StoredFile} storedFile
     */
    async storeFile(storedFile) {

        // Store file in database
        const file = await this.dbClient.createFile(storedFile)

        // Store file data in cache
        this.cache.commit(file.accessToken, file.fileData)

        // Persist file to remote storage
        this.tryPersistFile(storedFile)

        return file
    }

    /**
     * Persist a file to remote storage
     * 
     * @param {StoredFile} file
     */
    tryPersistFile(file) {
        this.storageProvider.uploadFile(file.uuid, file.fileData)
            .then(() => {

                // Persist file to remote storage
                const remoteUrl = this.storageProvider.buildUrl(file.uuid)
                return this.dbClient.setFileUrl(file.accessToken, remoteUrl)
            })
            .then(() => {

                // Evice file from cache
                this.cache.evict(file.accessToken)
            })
            .catch(err => {
                // Log error
                console.log(`[FileStorage/persist ${file.accessToken}] `, err)
                console.log(`[FileStorage/persist ${file.accessToken}] Trying again in 1s`)

                // Try again in 1 second
                setTimeout(() => this.tryPersistFile(file), 1000)
        })
    }

    /**
     * Retrieve a file
     * 
     * @param {string} accessToken
     * @returns {Promise<Prisma.File | null>}
     */
    async retrieveFile(accessToken) {
        const file = await this.dbClient.getFile(accessToken)
        
        if (file === null) {
            throw new Error('File not found')
        }

        // Check if file is only stored in cache
        if (file.dataUrl === null) {
            const fileData = this.cache.get(accessToken)

            // Patch file object
            file.fileData = fileData
        }

        return file
    }

    /**
     * Delete a file
     * 
     * @param {string} accessToken
     */
    async deleteFile(accessToken) {
        // Delete from database
        const file = await this.dbClient.deleteFile(accessToken)
        
        // Delete from remote storage
        if (file.dataUrl !== null) {
            await this.storageProvider.deleteFile(file.uuid)
        }

        // Delete from cache
        this.cache.evict(accessToken)
    }
}