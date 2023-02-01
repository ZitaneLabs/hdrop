import { v4 as uuidv4 } from 'uuid'
import Prisma from '@prisma/client'

import { TokenGenerator } from '../core.js'

type FileStorageLocationType = 'local' | 'remote'

export class ExportFileData {
    type: FileStorageLocationType
    
    _fileData: string | null
    _fileUrl: string | null

    constructor(type: FileStorageLocationType, fileData: string | null, fileUrl: string | null) {
        this.type = type
        this._fileData = fileData
        this._fileUrl = fileUrl
    }

    /**
     * Get base64 file data.
     */
    fileData(): string | null {
        return this._fileData
    }

    /**
     * Get remote file URL.
     */
    fileUrl(): string | null {
        return this._fileUrl
    }

    static fromFile(file: Prisma.File & Partial<{ fileData: string }>) {
        if (file.dataUrl !== null) {
            return this.fromRemote(file.dataUrl)
        }
        if (typeof file.fileData === 'string') {
            return this.fromLocal(file.fileData)
        }
        throw new Error('File data is not available')
    }

    static fromLocal(fileData: string) {
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
    fileData: string
    fileNameData: string
    fileNameHash: string
    salt: string
    iv: string

    /**
     * Construct a new `StoredFile` instance.
     */
    constructor({ fileData, fileNameData, fileNameHash, salt, iv }: {
            fileData: string
            fileNameData: string
            fileNameHash: string
            salt: string
            iv: string
        }) {
        // Generate UUID
        this.uuid = uuidv4()

        // Generate tokens
        this.accessToken = TokenGenerator.generateAccessToken()
        this.updateToken = TokenGenerator.generateUpdateToken()

        // Store data
        this.fileData = fileData
        this.fileNameData = fileNameData
        this.fileNameHash = fileNameHash
        this.salt = salt
        this.iv = iv
    }
}