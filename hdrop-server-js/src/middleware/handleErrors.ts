import { Request, Response, NextFunction } from 'express'

export default function(err: any, _req: Request, res: Response, _next: NextFunction) {
    console.error(err.stack)
    res.status(500).json({
        status: 'error',
        data: {
            reason: 'Internal server error'
        }
    })
}