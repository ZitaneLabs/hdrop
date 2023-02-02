import { Logger } from 'winston'
import { DatabaseClient, FileStorage } from '../core.js'

export default class AppContext {
    public dbClient: DatabaseClient
    public storage: FileStorage
    public logger: Logger

    constructor(dbClient: DatabaseClient, storage: FileStorage, logger: Logger) {
        this.dbClient = dbClient
        this.storage = storage
        this.logger = logger
    }
}