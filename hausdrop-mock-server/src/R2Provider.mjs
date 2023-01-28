import { DeleteObjectCommand, GetObjectCommand, PutObjectCommand, S3Client } from '@aws-sdk/client-s3'

export default class R2Provider {
    /**
     * @type {{
     * accountId: string,
     * accessKeyId: string,
     * secretAccessKey: string,
     * bucketName: string,
     * }}
     */
    creds

    /**
     * @type {S3Client}
     */
    client

    /**
     * Construct a new `R2Provider` instance.
     */
    constructor() {
        // Retrieve credentials from environment
        this.creds = {
            accountId: process.env.R2_ACCOUNT_ID,
            accessKeyId: process.env.R2_ACCESS_KEY_ID,
            secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
            bucketName: process.env.R2_BUCKET_NAME,
            publicUrl: process.env.R2_PUBLIC_URL,
        }

        // Build S3 endpoint
        const endpoint = `https://${this.creds.accountId}.r2.cloudflarestorage.com`

        // Construct S3 client
        this.client = new S3Client({
            endpoint,
            region: 'auto',
            credentials: {
                accessKeyId: this.creds.accessKeyId,
                secretAccessKey: this.creds.secretAccessKey,
            },
        })
    }

    /**
     * Build a URL for the given UUID.
     * 
     * @param {string} uuid
     * @returns {string} URL
     */
    buildUrl(uuid) {
        const sanitizedPublicUrl = this.creds.publicUrl.replace(/\/+$/, '')
        return `${sanitizedPublicUrl}/${uuid}`
    }

    async uploadFile(uuid, content) {
        const command = new PutObjectCommand({
            Bucket: this.creds.bucketName,
            Key: uuid,
            Body: content,
        })
        await this.client.send(command)
    }

    async downloadFile(uuid) {
        const command = new GetObjectCommand({
            Bucket: this.creds.bucketName,
            Key: uuid,
        })
        const response = await this.client.send(command)
        return await this.streamToString(response.Body)
    }

    async deleteFile(uuid) {
        const command = new DeleteObjectCommand({
            Bucket: this.creds.bucketName,
            Key: uuid,
        })
        await this.client.send(command)
    }

    streamToString(readableStream) {
        return new Promise((resolve, reject) => {
            const chunks = []
            readableStream.on('data', (data) => {
                chunks.push(data)
            })
            readableStream.on('end', () => {
                resolve(Buffer.concat(chunks).toString('utf8'))
            })
            readableStream.on('error', reject)
        })
    }
}