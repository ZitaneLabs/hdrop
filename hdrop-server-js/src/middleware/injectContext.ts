import { NextFunction, Request, Response } from 'express'

import { AppContext } from '../core.js'

export default function injectContext(context: AppContext) {
    return (req: Request, _res: Response, next: NextFunction) => {
        req.context = context
        next()
    }
}