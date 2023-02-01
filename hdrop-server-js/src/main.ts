import express from 'express'
import bodyParser from 'body-parser'
import cors from 'cors'
import * as dotenv from 'dotenv'

import { AppContext, DatabaseClient, FileStorage, S3Provider } from './core.js'
import { handleErrors, injectContext } from './middleware.js'
import { statusRouter, v1Router } from './api.js'

// Load environment variables
dotenv.config()

// Parse env variables
const PORT = parseInt(process.env.PORT) || 8080
const CORS_ORIGIN = process.env.CORS_ORIGIN || '*'

async function main() {
    // Create storage provider
    const storageProvider = new S3Provider()
    await storageProvider.setupCors()

    // Open database connection
    const dbClient = new DatabaseClient()

    // Create file storage
    const storage = new FileStorage(dbClient, storageProvider)

    // Create context
    const context = new AppContext(dbClient, storage)

    // Start expiration watchdog
    setInterval(() => {
        storage.purgeExpiredFiles()
    }, 60 * 1000)

    // Create express app
    const app = express()
    app.use(bodyParser.json({ limit: '1gb' }))
    app.use(cors({ origin: CORS_ORIGIN }))
    app.use(injectContext(context))
    app.use(handleErrors)

    app.use('/status', statusRouter)
    app.use('/v1', v1Router)

    app.listen(PORT, () => {
        console.log(`⚡️ Server is running at http://localhost:${PORT}`)
    })
}

main().catch(err => {
    console.error(err)
    process.exit(1)
})