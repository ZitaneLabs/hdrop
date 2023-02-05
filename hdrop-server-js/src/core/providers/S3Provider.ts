import { DeleteObjectCommand, PutObjectCommand, PutBucketCorsCommand, S3Client } from '@aws-sdk/client-s3'

export default class S3Provider {
    creds: {
        endpoint: string
        region: string
        accessKeyId: string
        secretAccessKey: string
        bucketName: string
        publicUrl: string
    }

    /**
     * @type {S3Client}
     */
    client: S3Client

    /**
     * Construct a new `S3Provider` instance.
     */
    constructor() {
        // Retrieve credentials from environment
        this.creds = {
            endpoint: process.env.S3_ENDPOINT,
            region: process.env.S3_REGION,
            accessKeyId: process.env.S3_ACCESS_KEY_ID,
            secretAccessKey: process.env.S3_SECRET_ACCESS_KEY as string,
            bucketName: process.env.S3_BUCKET_NAME as string,
            publicUrl: process.env.S3_PUBLIC_URL as string,
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
     */
    buildUrl(uuid: string): string {
        const sanitizedPublicUrl = this.creds.publicUrl.replace(/\/+$/, '')
        return `${sanitizedPublicUrl}/${uuid}`
    }

    async setupCors() {
        const command = new PutBucketCorsCommand({
            Bucket: this.creds.bucketName,
            CORSConfiguration: {
                CORSRules: [
                    {
                        AllowedOrigins: ['*'],
                        AllowedMethods: ['GET'],
                    }
                ]
            }
        })
        await this.client.send(command)
    }

    async uploadFile(uuid: string, content: Buffer) {
        const command = new PutObjectCommand({
            Bucket: this.creds.bucketName,
            Key: uuid,
            Body: content,
        })
        await this.client.send(command)
    }

    async deleteFile(uuid: string) {
        const command = new DeleteObjectCommand({
            Bucket: this.creds.bucketName,
            Key: uuid,
        })
        await this.client.send(command)
    }
}