import axios, { AxiosRequestConfig, AxiosResponse } from "axios"
import { EncryptedFileInfo } from "../util"

/**
 * Base URL of the website
 * 
 * @type {string}
 */
const HOMEPAGE = 'http://localhost:3000'

/**
 * Base URL of the API
 * 
 * @type {string}
 */
const API_BASE = 'http://localhost:8080'

/**
 * A function that does nothing.
 */
const NoopHandler = () => {}

/**
 * HausDrop API client
 */
class ApiClient {
    /**
     * @typedef {{ status: "success", data: object }} SuccessResponse
     * @typedef {{ status: "error", data: { reason: string }}} ErrorResponse
     * @typedef {SuccessResponse | ErrorResponse} ApiResponse
     * @param {AxiosResponse<ApiResponse>} resp
     */
    static processResponse(resp) {
        const json = resp.data
        switch (json.status) {
            case "success":
                return json.data
            case "error":
                throw new Error(json.data.reason)
            default:
                throw new Error("Unknown response status")
        }
    }

    /**
     * Build an API endpoint.
     * 
     * @param {string} path Path
     * @param {Record<string, string>} query Query parameters
     * @returns {string} The full endpoint
     */
    static buildEndpoint(path, query = {}) {
        const params = new URLSearchParams(query)
        const normalizedPath = path.replace(/^\/+/, '')
        return `${API_BASE}/${normalizedPath}?${params.toString()}`
    }

    /**
     * Upload a file to the server.
     * 
     * @typedef {{ access_token: string, update_token: string }} UploadFileResponse
     * @param {EncryptedFileInfo} fileInfo
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<UploadFileResponse>} Response data
     */
    static async uploadFile(fileInfo, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Prepare request data
        const data = {
            file_data: fileInfo.fileDataBase64(),
            file_name_data: fileInfo.fileNameDataBase64(),
            file_name_hash: fileInfo.challenge(),
            iv: fileInfo.ivBase64(),
            salt: fileInfo.saltBase64(),
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files`)
        const resp = await axios.post(endpoint, data, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    /**
     * Update the expiration time of a file.
     * 
     * @param {string} accessToken Access token
     * @param {string} updateToken Update token
     * @param {number} expirationS Expiration time in seconds
     * @param {(progress: number) => void} onProgressChange Progress change callback
     * @returns {Promise<void>}
     */
    static async updateFileExpiration(accessToken, updateToken, expirationS, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Prepare request data
        const data = {
            expiry: expirationS,
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/expiry`, { updateToken })
        const resp = await axios.post(endpoint, data, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    /**
     * Request a file challenge from the server.
     * 
     * @typedef {{ challenge: string, salt: string, iv: string, }} GetChallengeResponse
     * @param {string} accessToken Access token
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<GetChallengeResponse>} Response data
     */
    static async getChallenge(accessToken, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/challenge`)
        const resp = await axios.get(endpoint, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    /**
     * Submit a file challenge to the server.
     * 
     * @typedef {{ salt: string, iv: string, }} SubmitChallengeResponse
     * @param {string} accessToken Access token
     * @param {string} challengeSolution Challenge solution hash
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<SubmitChallengeResponse>} Response data
     */
    static async submitChallenge(accessToken, challengeSolution, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Prepare request data
        const data = {
            challenge: challengeSolution
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/challenge`)
        const resp = await axios.post(endpoint, data, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    /**
     * Get a file from the server.
     * 
     * @typedef {{ file_data: string, file_name_data: string, iv: string, salt: string }} GetFileResponse
     * @param {string} accessToken Access token
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<GetFileResponse>} Response data
     */
    static async getFile(accessToken, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}`)
        const resp = await axios.get(endpoint, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    /**
     * Delete a file from the server.
     * 
     * @param {string} accessToken Access token
     * @param {string} updateToken Update token
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<void>}
     */
    static async deleteFile(accessToken, updateToken, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onUploadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}`, { updateToken })
        const resp = await axios.delete(endpoint, config)

        // Process response
        return ApiClient.processResponse(resp)
    }

    static generateLink(accessToken) {
        return `${HOMEPAGE}/${accessToken}`
    }
}

export default ApiClient