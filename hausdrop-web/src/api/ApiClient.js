import axios, { AxiosRequestConfig } from "axios"
import { EncryptedFileInfo } from "../util"

const API_BASE = 'http://localhost:8080'

/**
 * HausDrop API client
 */
class ApiClient {

    /**
     * Uploads a file to the server
     * 
     * @param {EncryptedFileInfo} fileInfo
     * @param {(progress: number) => void} onProgress
     * @returns {Promise<number>} Response status code
     */
    static async uploadFile(fileInfo, onProgress) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgress(progressEvent.loaded)
            }
        }
        const data = {
            file_data: fileInfo.fileDataBase64(),
            file_name_data: fileInfo.fileNameDataBase64(),
            iv: fileInfo.ivBase64(),
            salt: fileInfo.saltBase64(),
        }
        const resp = await axios.put(`${API_BASE}/v1/file`, data, config)
        return resp.status
    }
}

export default ApiClient