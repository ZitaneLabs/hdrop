import { v4 as uuidv4 } from 'uuid'
import Prisma from '@prisma/client'

import { DatabaseClient, TokenGenerator } from '../core.js'

type FileStorageLocationType = 'local' | 'remote'

export class ExportFileData {
    type: FileStorageLocationType
    
    _fileData: Buffer | null
    _fileUrl: string | null

    constructor(type: FileStorageLocationType, fileData: Buffer | null, fileUrl: string | null) {
        this.type = type
        this._fileData = fileData
        this._fileUrl = fileUrl
    }

    /**
     * Get raw file data.
     */
    fileData(): Buffer | null {
        return this._fileData
    }

    /**
     * Get remote file URL.
     */
    fileUrl(): string | null {
        return this._fileUrl
    }

    static fromFile(file: Prisma.File & Partial<{ fileData: Buffer }>) {
        if (file.dataUrl !== null) {
            return this.fromRemote(file.dataUrl)
        }
        if (file.fileData instanceof Buffer) {
            return this.fromLocal(file.fileData)
        }
        throw new Error('File data is not available')
    }

    static fromLocal(fileData: Buffer) {
        return new ExportFileData('local', fileData, null)
    }

    static fromRemote(fileUrl: string) {
        return new ExportFileData('remote', null, fileUrl)
    }
}

export default class StoredFile {
    uuid: string
    accessToken: string
    updateToken: string
    fileData: Buffer
    fileNameData: string
    fileNameHash: string
    salt: string
    iv: string

    /**
     * Construct a new `StoredFile` instance.
     */
    constructor({ fileData, fileNameData, fileNameHash, salt, iv }: {
            fileData: Buffer
            fileNameData: string
            fileNameHash: string
            salt: string
            iv: string
        }) {
        // Generate UUID
        this.uuid = uuidv4()

        // Generate update token
        this.accessToken = TokenGenerator.generateFallbackAccessToken()
        this.updateToken = TokenGenerator.generateUpdateToken()

        // Store data
        this.fileData = fileData
        this.fileNameData = fileNameData
        this.fileNameHash = fileNameHash
        this.salt = salt
        this.iv = iv
    }

    async generateAccessToken(dbClient: DatabaseClient) {
        this.accessToken = await TokenGenerator.generateAccessToken(dbClient)
    }
}