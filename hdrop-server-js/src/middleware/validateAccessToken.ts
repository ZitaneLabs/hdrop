import { Request, Response, NextFunction } from 'express'

export default async function validateAccessToken(req: Request, res: Response, next: NextFunction) {
    const { accessToken } = req.params

    if (!await req.context.dbClient.accessTokenExists(accessToken)) {
        return res.status(404).json({
            reason: 'File not found'
        })
    }

    next()
}
