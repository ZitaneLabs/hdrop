import axios, { AxiosRequestConfig, AxiosResponse } from "axios"
import { EncryptedFileInfo } from "../util"

/**
 * Base URL of the website
 * 
 * @type {string}
 */
const HOMEPAGE = process.env.REACT_APP_BASE_URL

/**
 * Base URL of the API
 * 
 * @type {string}
 */
const API_BASE = process.env.REACT_APP_API_ENDPOINT

/**
 * A function that does nothing.
 */
const NoopHandler = () => {}

/**
 * hdrop API client
 */
class ApiClient {
    /**
     * @typedef {{ status: "success", data: object }} SuccessResponse
     * @typedef {{ status: "error", data: { reason: string }}} ErrorResponse
     * @typedef {SuccessResponse | ErrorResponse} ApiResponse
     * @param {AxiosResponse<ApiResponse>} resp
     */
    static processResponse(resp) {
        const data = resp.data
        switch (data.status) {
            case "success":
                return data.data
            case "error":
                throw new Error(data.data.reason)
            default:
                console.log(`Invalid status: "${data.status}"`)
                throw new Error("Unknown response status")
        }
    }

    /**
     * Wrap a request in a try-catch block.
     * 
     * @param {Promise<Record<string, any>>} fn
     * @param {{
     * skipValidation?: boolean,
     * }} _config
     */
    static async wrapRequest(request, _config = {}) {
        const config = {
            skipValidation: false,
            ..._config,
        }
        try {
            const resp = await request
            if (config.skipValidation) return resp.data
            else return this.processResponse(resp)
        } catch (err) {
            if ('response' in err && 'data' in err.response) {
                throw new Error(err.response.data.data.reason)
            } else {
                throw err
            }
        }
    }

    /**
     * Check if the server is online.
     */
    static async isServerOnline() {
        /** @type {AxiosRequestConfig} */
        const config = {
            responseType: 'text'
        }
        try {
            const res = await this.wrapRequest(axios.get(`${API_BASE}/status`, config), { skipValidation: true })
            return res === 'OK'
        } catch (_err) {
            return false
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
            },
            headers: {
                'Content-Type': 'multipart/form-data'
            }
        }

        // Build form data
        const formData = new FormData()
        formData.set('file_data', new Blob([fileInfo.fileData()]))
        formData.set('file_name_data', fileInfo.fileNameDataBase64())
        formData.set('file_name_hash', fileInfo.challenge())
        formData.set('iv', fileInfo.ivBase64())
        formData.set('salt', fileInfo.saltBase64())

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files`)
        return await this.wrapRequest(axios.post(endpoint, formData, config))
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
        return await this.wrapRequest(axios.post(endpoint, data, config))
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
            onDownloadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/challenge`)
        return await this.wrapRequest(axios.get(endpoint, config))
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
            onDownloadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Prepare request data
        const data = {
            challenge: challengeSolution
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/challenge`)
        return await this.wrapRequest(axios.post(endpoint, data, config))
    }

    /**
     * Get file info from the server.
     * 
     * @typedef {{ file_url: string | null, file_name_data: string, iv: string, salt: string }} GetFileResponse
     * @param {string} accessToken Access token
     * @param {(progress: number) => void} onProgressChange
     * @returns {Promise<GetFileResponse>} Response data
     */
    static async getFileInfo(accessToken, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            onDownloadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}`)
        return await this.wrapRequest(axios.get(endpoint, config))
    }

    /**
     * Download a file from the server.
     * 
     * @param {string} accessToken Access token
     * @param {(progress: number) => void} onProgressChange
     * 
     * @returns {Promise<ArrayBuffer>}
     */
    static async directDownloadFile(accessToken, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            responseType: 'arraybuffer',
            headers: {
                'Accept': 'application/octet-stream, */*',
            },
            onDownloadProgress: progressEvent => {
                console.log(progressEvent)
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Download file
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}/raw`)
        return await this.wrapRequest(axios.get(endpoint, config), {
            skipValidation: true
        })
    }

    /**
     * Download a file from an URL.
     * 
     * @param {string} url URL
     * @param {(progress: number) => void} onProgressChange
     * 
     * @returns {Promise<ArrayBuffer>}
     */
    static async directDownloadFileFromUrl(url, onProgressChange = NoopHandler) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {
            responseType: 'arraybuffer',
            headers: {
                'Accept': 'application/octet-stream, */*',
            },
            onDownloadProgress: progressEvent => {
                onProgressChange(progressEvent.loaded / progressEvent.total)
            }
        }

        // Download file
        return await this.wrapRequest(axios.get(url, config), {
            skipValidation: true
        })
    }

    /**
     * Delete a file from the server.
     * 
     * @param {string} accessToken Access token
     * @param {string} updateToken Update token
     * @returns {Promise<void>}
     */
    static async deleteFile(accessToken, updateToken) {
        /**
         * @type {AxiosRequestConfig}
         */
        const config = {}

        // Send request
        const endpoint = ApiClient.buildEndpoint(`/v1/files/${accessToken}`, { updateToken })
        return await this.wrapRequest(axios.delete(endpoint, config))
    }

    /**
     * Generate a link to a file.
     * 
     * @param {string} accessToken Access token
     * @param {string | null} password Password
     */
    static generateLink(accessToken, password = null) {
        const suffix = password !== null ? `#${password}` : ''
        return `${HOMEPAGE}/${accessToken}${suffix}`
    }
}

export default ApiClient