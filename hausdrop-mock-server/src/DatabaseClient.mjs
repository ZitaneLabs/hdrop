
import Prisma from '@prisma/client'
const { PrismaClient, File } = Prisma

import StoredFile from './StoredFile.mjs'

export default class DatabaseClient {
    client = new PrismaClient()

    /**
     * Create a new file in the database.
     * 
     * @param {StoredFile} storedFile
     * @returns {Promise<File>}
     */
    async createFile(storedFile) {
        return await this.client.file.create({
            data: {
                uuid: storedFile.uuid,
                accessToken: storedFile.accessToken,
                updateToken: storedFile.updateToken,
                fileNameData: storedFile.fileNameData,
                fileNameHash: storedFile.fileNameHash,
                salt: storedFile.salt,
                iv: storedFile.iv,
                createdAt: new Date(),
                expiresAt: new Date()
            }
        })
    }

    /**
     * Get the creation date of a file.
     * 
     * @param {string} accessToken
     * @returns {Promise<Date | null>}
     */
    async getFileCreationDate(accessToken) {
        const file = await this.client.file.findUnique({
            where: {
                accessToken
            },
            select: {
                createdAt: true
            }
        })

        return file?.createdAt ?? null
    }

    /**
     * Get the expiry date of a file.
     * 
     * @param {string} accessToken
     * @returns {Promise<Date | null>}
     */
    async getFileExpiry(accessToken) {
        const file = await this.client.file.findUnique({
            where: {
                accessToken
            },
            select: {
                expiresAt: true
            }
        })

        return file?.expiresAt ?? null
    }

    /**
     * Set the expiry date of a file.
     * 
     * @param {string} accessToken
     * @param {number} expirySeconds
     */
    async setFileExpiry(accessToken, expirySeconds) {
        const creationDate = await this.getFileCreationDate(accessToken)
        if (creationDate === null) {
            throw new Error('File not found')
        }
        const expiresAt = new Date(creationDate.getTime() + expirySeconds * 1000)
        await this.client.file.update({
            where: {
                accessToken,
            },
            data: {
                expiresAt
            }
        })
    }

    /**
     * Set the data URL of a file.
     * 
     * @param {string} accessToken
     * @param {string} url
     */
    async setFileUrl(accessToken, url) {
        await this.client.file.update({
            where: {
                accessToken,
            },
            data: {
                dataUrl: url
            }
        })
    }

    /**
     * Find a file by its access token.
     * 
     * @param {string} accessToken
     * @returns {Promise<File | null>}
     */
    async getFile(accessToken) {
        return await this.client.file.findUnique({
            where: {
                accessToken
            }
        })
    }

    /**
     * Delete a file by its access token.
     * 
     * @param {string} accessToken
     * @returns {Promise<File>}
     */
    async deleteFile(accessToken) {
        return await this.client.file.delete({
            where: {
                accessToken
            }
        })
    }
}