import { Request, Response, NextFunction } from 'express'

export default async function authenticated(req: Request, res: Response, next: NextFunction) {
    const { updateToken } = req.query
    const { accessToken } = req.params as { accessToken?: string }

    // Check for access token
    if (typeof accessToken !== 'string') {
        return res.status(400).json({
            status: 'error',
            data: {
                reason: 'Missing access token'
            }
        })
    }

    // Check for update token
    if (typeof updateToken !== 'string') {
        return res.status(400).json({
            status: 'error',
            data: {
                reason: 'Missing update token'
            }
        })
    }

    // Validate tokens
    if (!await req.context.dbClient.validateTokens(accessToken, updateToken)) {
        return res.status(401).json({
            status: 'error',
            data: {
                reason: 'Invalid access- or update-token'
            }
        })
    }

    next()
}