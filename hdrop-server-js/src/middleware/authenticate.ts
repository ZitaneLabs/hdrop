import { Request, Response, NextFunction } from 'express'

export default function authenticate(req: Request, res: Response, next: NextFunction) {
    const { updateToken } = req.query

    if (typeof updateToken !== 'string') {
        return res.status(400).json({
            status: 'error',
            data: {
                reason: 'Missing update token'
            }
        })
    }

    next()
}