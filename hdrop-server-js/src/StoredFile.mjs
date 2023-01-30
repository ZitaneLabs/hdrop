import { v4 as uuidv4 } from 'uuid'
import Prisma from '@prisma/client'

import TokenGenerator from './TokenGenerator.mjs'

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
     * @param {Prisma.File & { fileData: string | undefined }} file
     */
    static fromFile(file) {
        if (file.dataUrl !== null) {
            return this.fromRemote(file.dataUrl)
        }
        return this.fromLocal(file.fileData)
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
    /** @type {string} */
    uuid

    /** @type {string} */
    accessToken

    /** @type {string} */
    updateToken

    /** @type {string} */
    fileData

    /** @type {string} */
    fileNameData

    /** @type {string} */
    fileNameHash

    /** @type {string} */
    salt

    /** @type {string} */
    iv

    /**
     * Construct a new `StoredFile` instance.
     * 
     * @param {{
     * fileData: string,
     * fileNameData: string,
     * fileNameHash: string,
     * salt: string,
     * iv: string,
     * }} data
     */
    constructor({ fileData, fileNameData, fileNameHash, salt, iv }) {
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