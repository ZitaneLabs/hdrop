import { Request, Response, Router } from 'express'

import { authenticate } from '../../middleware.js'
import { StoredFile, ExportFileData } from '../../core.js'

const router = Router()

router.post('/', async (req: Request, res: Response) => {
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

    // Generate access token
    await fileBody.generateAccessToken(req.context.dbClient)

    // Store file
    const file = await req.context.storage.storeFile(fileBody)

    // Send response
    res.json({
        status: 'success',
        data: {
            access_token: file.accessToken,
            update_token: file.updateToken,
        }
    })
})

router.post('/:accessToken/expiry', authenticate, async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

    // Guard against invalid access token
    if (file === null) {
        return res.status(404).json({
            status: 'error',
            data: {
                reason: 'File not found'
            }
        })
    }

    // Update file expiry
    await req.context.dbClient.setFileExpiry(accessToken, req.body.expiry)

    // Send response
    res.json({
        status: 'success',
        data: {}
    })
})

router.get('/:accessToken', async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

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

router.get('/:accessToken/challenge', async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

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

router.post('/:accessToken/challenge', async (req: Request, res: Response) => {
    const { accessToken } = req.params

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

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

router.delete('/:accessToken', authenticate, async (req: Request, res: Response) => {
    const { accessToken } = req.params
    const { update_token: updateToken } = req.query as { update_token: string }

    // Retrieve file
    const file = await req.context.storage.retrieveFile(accessToken)

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
    if (updateToken !== file.updateToken) {
        return res.status(403).json({
            status: 'error',
            data: {
                reason: 'Invalid update token'
            }
        })
    }

    // Delete file
    await req.context.storage.deleteFile(accessToken)

    // Send response
    res.json({
        status: 'success',
    })
})

export default router