import AppContext from "../AppContext.js"
import DatabaseClient from "./DatabaseClient.mjs"
import FileStorage from "./FileStorage.mjs"

declare global {
    namespace Express {
        interface Request {
            context: AppContext
        }
    }
}

export {}