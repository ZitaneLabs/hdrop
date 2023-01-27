import express from 'express'
import bodyParser from 'body-parser'
import cors from 'cors'
import { v4 as uuidv4 } from 'uuid'
import { createHash } from 'crypto'

const app = express()
app.use(bodyParser.json({ limit: '10gb' }))
app.use(cors())

const files = {}

const _generateToken = () => {
    const uuid = uuidv4()
    const sha256 = createHash('sha256').update(uuid).digest('hex')
    const token = sha256.substring(0, 8)
    return token
}

const generateAccessToken = () => {
    const token = _generateToken()
    if (token in files) {
        return generateAccessToken()
    }
    return token
}

const generateUpdateToken = () => {
    return _generateToken()
}

app.post('/proxy', (req, res) => {
    console.log(req.body)
    res.json(req.body)
})

app.post('/v1/files', (req, res) => {
    // Generate tokens
    const accessToken = generateAccessToken()
    const updateToken = generateUpdateToken()

    // Store file
    files[accessToken] = {
        ...req.body,
        accessToken,
        updateToken,
    }

    // Send response
    res.json({
        status: 'success',
        data: {
            access_token: accessToken,
            update_token: updateToken,
        }
    })
})

app.get('/v1/files/:access_token', (req, res) => {
    // Retrieve file
    const file = files[req.params.access_token]

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
            file_data: file.file_data,
            file_name_data: file.file_name_data,
            iv: file.iv,
            salt: file.salt,
        }
    })
})

app.get('/v1/files/:access_token/challenge', (req, res) => {
    // Retrieve file
    const file = files[req.params.access_token]

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
            challenge: file.file_name_data,
            iv: file.iv,
            salt: file.salt,
        }
    })
})

app.post('/v1/files/:access_token/challenge', (req, res) => {
    // Retrieve file
    const file = files[req.params.access_token]

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
    if (req.body.challenge !== file.file_name_hash) {
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

app.delete('/v1/files/:access_token', (req, res) => {
    // Retrieve file
    const file = files[req.params.access_token]

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
    delete files[req.params.access_token]

    // Send response
    res.json({
        status: 'success',
    })
})

app.listen(8080)