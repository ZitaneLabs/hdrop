import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

export default class TokenGenerator {
    /**
     * @type {string[]}
     */
    static accessTokens = []

    /**
     * Access token length
     * 
     * @type {number}
     */
    static accessTokenLength = 5

    /**
     * Update token length
     * 
     * @type {number}
     */
    static updateTokenLength = 8

    static generateToken(length) {
        const uuid = uuidv4()
        const sha256 = createHash('sha256').update(uuid).digest('hex')
        const token = sha256.substring(0, length)
        return token
    }
    
    /**
     * Generate a unique access token
     * 
     * @returns {string}
     */
    static generateAccessToken() {
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
     * 
     * @returns {string}
     */
    static generateUpdateToken() {
        return this.generateToken(this.updateTokenLength)
    }
}