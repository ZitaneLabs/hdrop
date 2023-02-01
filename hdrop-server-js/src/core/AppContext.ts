import { DatabaseClient, FileStorage } from '../core.js'

export default class AppContext {
    dbClient: DatabaseClient
    storage: FileStorage

    constructor(dbClient: DatabaseClient, storage: FileStorage) {
        this.dbClient = dbClient
        this.storage = storage
    }
}