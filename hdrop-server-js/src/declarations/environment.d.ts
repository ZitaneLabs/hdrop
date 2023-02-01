declare global {
    namespace NodeJS {
        interface ProcessEnv {
            PORT: string
            S3_REGION: string
            S3_ENDPOINT: string
            S3_ACCESS_KEY_ID: string
            S3_SECRET_ACCESS_KEY: string | undefined
            S3_BUCKET_NAME: string | undefined
            S3_PUBLIC_URL: string | undefined
            CORS_ORIGIN: string | undefined
        }
    }
}

export {}