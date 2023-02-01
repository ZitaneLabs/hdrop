import DatabaseClient from "./DatabaseClient.mjs";
import FileStorage from "./FileStorage.mjs";

export default class AppContext {
    dbClient: DatabaseClient
    storage: FileStorage

    constructor(dbClient: DatabaseClient, storage: FileStorage) {
        this.dbClient = dbClient
        this.storage = storage
    }
}