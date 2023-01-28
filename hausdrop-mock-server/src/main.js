import express from 'express'
import bodyParser from 'body-parser'
import cors from 'cors'
import * as dotenv from 'dotenv'

import R2Provider from './R2Provider.mjs'
import FileStorage from './FileStorage.mjs'
import StoredFile from './StoredFile.mjs'

// Load environment variables
dotenv.config()

// Create R2 provider
const r2 = new R2Provider()

// Create file storage
const storage = new FileStorage()

// Create express app
const app = express()
app.use(bodyParser.json({ limit: '10gb' }))
app.use(cors())

app.post('/proxy', (req, res) => {
    console.log(req.body)
    res.json(req.body)
})

app.post('/v1/files', (req, res) => {
    // Create stored file
    const file = new StoredFile(
        r2,
        {
            fileData: req.body.file_data,
            fileNameData: req.body.file_name_data,
            fileNameHash: req.body.file_name_hash,
            salt: req.body.salt,
            iv: req.body.iv,
        }
    )

    // Store file
    storage.storeFile(file)

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
    const file = storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (!file) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Export file data
    const fileData = await file.exportFileData();

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

    // Print cache statistics
    StoredFile.printCacheStatistics()

    // Send response
    res.json(data)
})

app.get('/v1/files/:access_token/challenge', (req, res) => {
    // Retrieve file
    const file = storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (!file) {
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

app.post('/v1/files/:access_token/challenge', (req, res) => {
    // Retrieve file
    const file = storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (!file) {
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
    const file = storage.retrieveFile(req.params.access_token)

    // Guard against invalid access token
    if (!file) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Guard against invalid update token
    if (req.query('update_token') !== file.update_token) {
        return res.status(403).json({
            status: 'error',
            data: {
                reason: 'Invalid update token'
            }
        })
    }

    // Delete file
    await storage.deleteFile(req.params.access_token, r2)

    // Send response
    res.json({
        status: 'success',
    })
})

app.listen(8080)