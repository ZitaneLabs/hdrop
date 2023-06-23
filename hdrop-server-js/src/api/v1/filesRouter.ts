import { Request, Response, Router } from 'express'
import multer from 'multer'

import { authenticated } from '../../middleware.js'
import { StoredFile } from '../../core.js'
import { Readable } from 'stream'

const upload = multer({
    storage: multer.memoryStorage(),
    limits: {
        fileSize: 256 * 1024 * 1024, // 256 MiB
    }
})
const router = Router()

router.post('/', upload.single('file_data'), async (req: Request, res: Response) => {
    const fileData = req.file

    if (fileData === undefined) {
        return res.status(400).json({
            reason: 'No file data provided'
        })
    }

    // Create stored file
    const fileBody = new StoredFile(
        {
            fileData: fileData.buffer,
            fileNameData: req.body.file_name_data,
            challengeData: req.body.challenge_data,
            challengeHash: req.body.challenge_hash,
            salt: req.body.salt,
            iv: req.body.iv,
        }
    )

    // Generate access token
    await fileBody.generateAccessToken(req.context.dbClient)

    // Store file
    const file = await req.context.storage.storeFile(fileBody)

    // Send response
    res.json({
        access_token: file.accessToken,
        update_token: file.updateToken,
    })
})

router.post('/:accessToken/expiry', authenticated, async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    // Update file expiry
    await req.context.dbClient.setFileExpiry(accessToken, req.body.expiry)

    // Send response
    res.json({})
})

router.get('/:accessToken', async (req: Request, res: Response) => {
    const { accessToken } = req.params
    const challengeHash = (req.headers.authorization ?? '').split(' ')[1]

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    // Authorize user
    if (file.challengeHash !== challengeHash) {
        return res.status(403).json({
            reason: 'Invalid challenge'
        })
    }

    if (file.dataUrl !== null) {

        // Send response
        return res.json({
            file_url: file.dataUrl,
        })
    }

    // Retrieve raw file data
    const fileData = req.context.storage.retrieveRawFileData(accessToken)

    // Guard against expired or cache-evicted file
    if (!fileData) {
        return res.status(404).json({
            reason: 'Raw file data is not available anymore'
        })
    }

    // Stream file data to client
    res.writeHead(200, {
        'Content-Type': 'application/octet-stream',
        'Content-Length': fileData.byteLength,
    })
    const stream = Readable.from(fileData)
    stream.pipe(res)
})

router.get('/:accessToken/challenge', async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    // Send response
    res.json({
        challenge: file.challengeData,
        iv: file.iv,
        salt: file.salt,
    })
})

router.post('/:accessToken/challenge', async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    // Guard against invalid challenge
    if (req.body.challenge !== file.challengeHash) {
        return res.status(403).json({
            reason: 'Invalid challenge'
        })
    }

    // Send response
    res.json({
        file_name_data: file.fileNameData,
    })
})

router.delete('/:accessToken', authenticated, async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    // Delete file
    await req.context.storage.deleteFile(accessToken)

    // Send response
    res.json({})
})

export default router
