import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'
import DatabaseClient from './DatabaseClient.js'

export default class TokenGenerator {
    static accessTokens: string[] = []
    static accessTokenMinLength: number = 5
    static updateTokenLength: number = 8

    static generateToken(length: number) {
        const uuid = uuidv4()
        const sha256 = createHash('sha256').update(uuid).digest('hex')
        const token = sha256.substring(0, length)
        return token
    }
    
    /**
     * Generate a unique access token
     */
    static async generateAccessToken(dbClient: DatabaseClient): Promise<string> {
        let targetLength = this.accessTokenMinLength
        let token = this.generateToken(targetLength)
        let collisions = 0

        while (await dbClient.accessTokenExists(token)) {
            collisions += 1

            // Increase token length if there are too many collisions
            if (collisions > 10) {
                targetLength += 1
            }

            // Generate new token
            token = this.generateToken(targetLength)
        }

        return token
    }

    /**
     * Generate a fallback access token
     */
    static generateFallbackAccessToken(): string {
        return this.generateToken(15)
    }
    
    /**
     * Generate an update token
     */
    static generateUpdateToken(): string {
        return this.generateToken(this.updateTokenLength)
    }
}