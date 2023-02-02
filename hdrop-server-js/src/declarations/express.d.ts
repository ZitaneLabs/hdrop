import { AppContext, DatabaseClient, FileStorage } from '../core.js'

declare global {
    namespace Express {
        interface Request {
            context: AppContext
        }
    }
}

export {}