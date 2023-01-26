const express = require('express')
const bodyParser = require('body-parser')
const cors = require('cors')

const app = express()
app.use(bodyParser.json())
app.use(cors())

const ACCESS_TOKEN = 'abc123'
const UPDATE_TOKEN = 'def456'

app.post('/proxy', (req, res) => {
    console.log(req.body)
    res.json(req.body)
})

app.post('/v1/files', (req, res) => {
    res.json({
        status: 'success',
        data: {
            access_token: ACCESS_TOKEN,
            update_token: UPDATE_TOKEN,
        }
    })
})

app.get('/v1/files/:access_token', (req, res) => {
    // Guard against invalid access token
    if (req.params.access_token !== ACCESS_TOKEN) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }
    res.json({
        status: 'success',
        data: {
            file_data: "",
            file_name_data: "",
            iv: "",
            salt: "",
        }
    })
})

app.get('/v1/files/:access_token/metadata', (req, res) => {
    // Guard against invalid access token
    if (req.params.access_token !== ACCESS_TOKEN) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }
    res.json({
        status: 'success',
        data: {
            file_name_data: "",
            iv: "",
            salt: "",
        }
    })
})

app.delete('/v1/files/:access_token', (req, res) => {
    // Guard against invalid access token
    if (req.params.access_token !== ACCESS_TOKEN) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }
    // Guard against invalid update token
    if (req.query('update_token') !== UPDATE_TOKEN) {
        return res.status(403).json({
            status: 'error',
            data: {
                reason: 'Invalid update token'
            }
        })
    }
    res.json({
        status: 'success',
    })
})

app.listen(8080)