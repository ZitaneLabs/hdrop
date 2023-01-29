import { DeleteObjectCommand, GetObjectCommand, PutObjectCommand, S3Client } from '@aws-sdk/client-s3'

export default class S3Provider {
    /**
     * @type {{
     * endpoint: string,
     * region: string,
     * accessKeyId: string,
     * secretAccessKey: string,
     * bucketName: string,
     * publicUrl: string,
     * }}
     */
    creds

    /**
     * @type {S3Client}
     */
    client

    /**
     * Construct a new `S3Provider` instance.
     */
    constructor() {
        // Retrieve credentials from environment
        this.creds = {
            endpoint: process.env.S3_ENDPOINT,
            region: process.env.S3_REGION,
            accessKeyId: process.env.S3_ACCESS_KEY_ID,
            secretAccessKey: process.env.S3_SECRET_ACCESS_KEY,
            bucketName: process.env.S3_BUCKET_NAME,
            publicUrl: process.env.S3_PUBLIC_URL,
        }

        // Construct S3 client
        this.client = new S3Client({
            endpoint: this.creds.endpoint,
            region: this.creds.region,
            credentials: {
                accessKeyId: this.creds.accessKeyId,
                secretAccessKey: this.creds.secretAccessKey,
            },
            // Workaround for docker-compose setup
            forcePathStyle: true,
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