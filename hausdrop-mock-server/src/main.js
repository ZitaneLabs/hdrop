import express from 'express'
import bodyParser from 'body-parser'
import cors from 'cors'
import * as dotenv from 'dotenv'

import { S3Provider } from './providers/index.mjs'
import FileStorage from './FileStorage.mjs'
import StoredFile, { ExportFileData } from './StoredFile.mjs'
import DatabaseClient from './DatabaseClient.mjs'

// Load environment variables
dotenv.config()

// Parse env variables
const PORT = parseInt(process.env.PORT) || 8080
const CORS_ORIGIN = process.env.CORS_ORIGIN || '*'

// Create storage provider
const storageProvider = new S3Provider()
await storageProvider.setupCors()

// Open database connection
const dbClient = new DatabaseClient()

// Create file storage
const storage = new FileStorage(dbClient, storageProvider)

// Create express app
const app = express()
app.use(bodyParser.json({ limit: '1gb' }))
app.use(cors({ origin: CORS_ORIGIN }))

// Custom error handler
app.use((err, _req, res, _next) => {
    console.error(err.stack)
    res.status(500).json({
        status: 'error',
        data: {
            reason: 'Internal server error'
        }
    })
})

app.get('/status', (_req, res) => {
    res.send('OK')
})

app.post('/v1/files', async (req, res) => {
    // Create stored file
    const fileBody = new StoredFile(
        {
            fileData: req.body.file_data,
            fileNameData: req.body.file_name_data,
            fileNameHash: req.body.file_name_hash,
            salt: req.body.salt,
            iv: req.body.iv,
        }
    )

    // Store file
    const file = await storage.storeFile(fileBody)

    // Send response
    res.json({
        status: 'success',
        data: {
            access_token: file.accessToken,
            update_token: file.updateToken,
        }
    })
})

app.get('/v1/files/:access_token', async (req, res) => {
    // Retrieve file
    const file = await storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Export file data
    const fileData = ExportFileData.fromFile(file)

    // Build response data
    const data = {
        status: 'success',
        data: {
            file_data: fileData.fileData(),
            file_url: fileData.fileUrl(),
            file_name_data: file.fileNameData,
            iv: file.iv,
            salt: file.salt,
        }
    }

    // Send response
    res.json(data)
})

app.get('/v1/files/:access_token/challenge', async (req, res) => {
    // Retrieve file
    const file = await storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Send response
    res.json({
        status: 'success',
        data: {
            challenge: file.fileNameData,
            iv: file.iv,
            salt: file.salt,
        }
    })
})

app.post('/v1/files/:access_token/challenge', async (req, res) => {
    // Retrieve file
    const file = await storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Guard against invalid challenge
    if (req.body.challenge !== file.fileNameHash) {
        return res.status(403).json({
            status: 'error',
            data: {
                reason: 'Invalid challenge'
            }
        })
    }

    // Send response
    res.json({
        status: 'success',
        data: {
            iv: file.iv,
            salt: file.salt,
        }
    })
})

app.delete('/v1/files/:access_token', async (req, res) => {
    // Retrieve file
    const file = await storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Guard against invalid update token
    if (req.query('update_token') !== file.updateToken) {
        return res.status(403).json({
            status: 'error',
            data: {
                reason: 'Invalid update token'
            }
        })
    }

    // Delete file
    await storage.deleteFile(req.params.access_token)

    // Send response
    res.json({
        status: 'success',
    })
})

app.listen(PORT)