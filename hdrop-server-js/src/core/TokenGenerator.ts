import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

export default class TokenGenerator {
    static accessTokens: string[] = []
    static accessTokenLength: number = 5
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
    static generateAccessToken(): string {
        let token = this.generateToken(this.accessTokenLength)
        let collisions = 0

        while (this.accessTokens.includes(token)) {
            collisions += 1

            // Increase token length if there are too many collisions
            if (collisions > 50) {
                this.accessTokenLength += 1
            }

            // Generate new token
            token = this.generateToken(this.accessTokenLength)
        }

        return token
    }
    
    /**
     * Generate an update token
     */
    static generateUpdateToken(): string {
        return this.generateToken(this.updateTokenLength)
    }
}