import { v4 as uuidv4 } from 'uuid'

import { DatabaseClient, TokenGenerator } from '../core.js'

export default class StoredFile {
    uuid: string
    accessToken: string
    updateToken: string
    fileData: Buffer
    fileNameData: string
    challengeData: string
    challengeHash: string
    salt: string
    iv: string

    /**
     * Construct a new `StoredFile` instance.
     */
    constructor({ fileData, fileNameData, challengeData, challengeHash, salt, iv }: {
            fileData: Buffer
            fileNameData: string
            challengeData: string
            challengeHash: string
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
        this.challengeData = challengeData
        this.challengeHash = challengeHash
        this.salt = salt
        this.iv = iv
    }

    async generateAccessToken(dbClient: DatabaseClient) {
        this.accessToken = await TokenGenerator.generateAccessToken(dbClient)
    }
}
