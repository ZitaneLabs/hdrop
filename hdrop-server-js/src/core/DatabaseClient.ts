
import Prisma, { PrismaClient } from '@prisma/client'

import { StoredFile } from '../core.js'

export default class DatabaseClient {
    /**
     * Default expiration time in milliseconds.
     * 
     * @default 86400000 // 24 hours
     */
    static DEFAULT_EXPIRATION_MS: number = 24 * 60 * 60 * 1000

    client: PrismaClient = new PrismaClient()

    /**
     * Create a new file in the database.
     */
    async createFile(storedFile: StoredFile): Promise<Prisma.File> {
        const createdAt = new Date()
        const expiresAt = new Date(createdAt.getTime() + DatabaseClient.DEFAULT_EXPIRATION_MS)
        return await this.client.file.create({
            data: {
                uuid: storedFile.uuid,
                accessToken: storedFile.accessToken,
                updateToken: storedFile.updateToken,
                fileNameData: storedFile.fileNameData,
                fileNameHash: storedFile.fileNameHash,
                salt: storedFile.salt,
                iv: storedFile.iv,
                createdAt,
                expiresAt,
            }
        })
    }

    /**
     * Get the creation date of a file.
     */
    async getFileCreationDate(accessToken: string): Promise<Date | null> {
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
     */
    async getFileExpiry(accessToken: string): Promise<Date | null> {
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
     */
    async setFileExpiry(accessToken: string, expirySeconds: number) {
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
     */
    async setFileUrl(accessToken: string, url: string) {
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
     */
    async getFile(accessToken: string): Promise<Prisma.File | null> {
        return await this.client.file.findUnique({
            where: {
                accessToken
            }
        })
    }

    /**
     * Delete a file by its access token.
     */
    async deleteFile(accessToken: string): Promise<Prisma.File> {
        return await this.client.file.delete({
            where: {
                accessToken
            }
        })
    }

    /**
     * Get all expired files.
     */
    async getExpiredFiles(): Promise<Prisma.File[]> {
        return await this.client.file.findMany({
            where: {
                expiresAt: {
                    lte: new Date()
                }
            }
        })
    }

    async accessTokenExists(accessToken: string): Promise<boolean> {
        const count = await this.client.file.count({
            where: {
                accessToken
            },
        })

        return count !== 0
    }
}