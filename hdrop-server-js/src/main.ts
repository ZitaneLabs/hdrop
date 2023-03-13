import express from 'express'
import bodyParser from 'body-parser'
import cors from 'cors'
import winston from 'winston'
import * as dotenv from 'dotenv'
import PrometheusClient from 'prom-client'
import promBundle from 'express-prom-bundle'

import { AppContext, DatabaseClient, FileStorage, S3Provider } from './core.js'
import { handleErrors, injectContext } from './middleware.js'
import { statusRouter, v1Router } from './api.js'

// Load environment variables
dotenv.config()

// Parse env variables
const PORT = parseInt(process.env.PORT) || 8080
const CORS_ORIGIN = process.env.CORS_ORIGIN || '*'

// Set up prometheus
PrometheusClient.collectDefaultMetrics()

// Create logger
const logger = winston.createLogger({
    level: 'info',
    format: winston.format.combine(
        winston.format.colorize(),
        winston.format.json()
    ),
    transports: [
        new winston.transports.Console({
            format: winston.format.simple()
        })
    ]
})

async function main() {
    // Create storage provider
    const storageProvider = new S3Provider()
    await storageProvider.setupCors()

    // Open database connection
    const dbClient = new DatabaseClient()

    // Create file storage
    const storage = new FileStorage(dbClient, storageProvider)

    // Create context
    const context = new AppContext(dbClient, storage, logger)

    // Create express metrics middleware
    const metricsMiddleware = promBundle({
        includeUp: true,
        includeMethod: true,
    })

    // Start expiration watchdog
    setInterval(() => {
        storage.purgeExpiredFiles()
    }, 60 * 1000)

    // Create express app
    const app = express()
    app.use(bodyParser.json({ limit: '256mb' }))
    app.use(cors({ origin: CORS_ORIGIN }))
    app.use(injectContext(context))
    app.use(metricsMiddleware)
    app.use(handleErrors)

    app.use('/status', statusRouter)
    app.use('/v1', v1Router)

    app.listen(PORT, () => {
        logger.info(`Server is running at http://localhost:${PORT}`)
    })
}

main().catch(err => {
    logger.error(err)
    process.exit(1)
})