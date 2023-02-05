import Prisma, { File } from '@prisma/client'

import { DatabaseClient, FileDataCache, S3Provider, StoredFile } from '../core.js'

/**
 * File storage coordinator.
 * 
 * Handles database, remote storage and caching.
 */
export default class FileStorage {
    dbClient: DatabaseClient
    storageProvider: S3Provider
    cache: FileDataCache = new FileDataCache()

    constructor(dbClient: DatabaseClient, storageProvider: S3Provider) {
        this.dbClient = dbClient
        this.storageProvider = storageProvider
    }

    async storeFile(storedFile: StoredFile) {

        // Store file in database
        const file = await this.dbClient.createFile(storedFile)

        // Store file data in cache
        this.cache.commit(file.accessToken, storedFile.fileData)

        // Persist file to remote storage
        this.tryPersistFile(storedFile)

        return file
    }

    tryPersistFile(file: StoredFile) {
        this.storageProvider.uploadFile(file.uuid, file.fileData)
            .then(() => {

                // Persist file to remote storage
                const remoteUrl = this.storageProvider.buildUrl(file.uuid)
                return this.dbClient.setFileUrl(file.accessToken, remoteUrl)
            })
            .then(() => {

                // Evict file from cache in 5 minutes
                this.cache.evictAfter(file.accessToken, 1000 * 60 * 5)
            })
            .catch((err: any) => {
                // Log error
                console.log(`[FileStorage/persist ${file.accessToken}] `, err)
                console.log(`[FileStorage/persist ${file.accessToken}] Trying again in 1s`)

                // Try again in 1 second
                setTimeout(() => this.tryPersistFile(file), 1000)
        })
    }

    /**
     * Retrieve a file
     */
    async retrieveFile(accessToken: string): Promise<File | null> {
        const file = await this.dbClient.getFile(accessToken)
        
        if (file === null) {
            throw new Error('File not found')
        }

        return file
    }

    /**
     * Retrieve raw file data
     */
    retrieveRawFileData(accessToken: string): Buffer | null {
        return this.cache.get(accessToken)
    }

    /**
     * Delete a file
     */
    async deleteFile(accessToken: string, reason: string | null = null) {
        // Delete from database
        const file = await this.dbClient.deleteFile(accessToken)
        
        // Delete from remote storage
        if (file.dataUrl !== null) {
            await this.storageProvider.deleteFile(file.uuid)
        }

        // Delete from cache
        this.cache.evictImmediately(accessToken)

        // Log deletion
        console.log(`[FileStorage/delete ${accessToken}] Deleted file ("${reason ?? "User request"}")`)
    }

    /**
     * Purge expired files
     */
    async purgeExpiredFiles() {
        const expiredFiles = await this.dbClient.getExpiredFiles()
        for (const file of expiredFiles) {
            await this.deleteFile(file.accessToken, 'Expired')
        }
    }
}