import Axios, { AxiosRequestConfig, AxiosResponse, isAxiosError } from "axios"

import {
    DownloadFileJsonResponse,
    GetChallengeResponse,
    SubmitChallengeResponse,
    UploadFileResponse,
    UploadFileData,
} from "./"

/**
 * Base URL of the website
 *
 * @type {string}
 */
const HOMEPAGE = process.env.NEXT_PUBLIC_WEB_BASE_URL!

/**
 * Base URL of the API
 *
 * @type {string}
 */
const API_BASE: string = process.env.NEXT_PUBLIC_API_BASE_URL!

type ErrorResponse = {
    reason?: string
}

type ProgressHandler = (progress: number) => void
const Identity = () => void 0

export default class APIClient {
    static async send<T>(config: AxiosRequestConfig): Promise<T> {
        try {
            return (await Axios.request(config)).data as T
        } catch (e) {
            if (isAxiosError(e)) {
                const errorResponse = (e.response?.data ?? {}) as ErrorResponse
                throw new Error(errorResponse.reason ?? e.message)
            }
            throw e
        }
    }

    static async sendRaw<T>(config: AxiosRequestConfig): Promise<AxiosResponse<T>> {
        try {
            return await Axios.request(config)
        } catch (e) {
            if (isAxiosError(e)) {
                const errorResponse = (e.response?.data ?? {}) as ErrorResponse
                throw new Error(errorResponse.reason ?? e.message)
            }
            throw e
        }
    }

    /** Build an API endpoint. */
    static buildEndpoint(path: string | Array<string>, query: Record<string, string> = {}): string {
        const params = new URLSearchParams(query)
        const normalizedPath = (Array.isArray(path) ? path.join('/') : path).replace(/^\/+/, '')
        return `${API_BASE}/${normalizedPath}?${params.toString()}`
    }

    static async uploadFile(data: UploadFileData, onProgressChange: ProgressHandler = Identity): Promise<UploadFileResponse> {
        return await this.send({
            method: 'POST',
            data: data.toMultipart(),
            url: this.buildEndpoint(['v1', 'files']),
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'multipart/form-data'
            },
            onUploadProgress(progressEvent) {
                if (progressEvent.total === undefined) return;
                onProgressChange(progressEvent.loaded / progressEvent.total)
            },
        })
    }

    static async downloadFile(accessToken: string, challengeHash: string, onProgressChange: ProgressHandler = Identity): Promise<ArrayBuffer> {
        const response = await this.sendRaw<ArrayBuffer>({
            method: 'GET',
            url: this.buildEndpoint(['v1', 'files', accessToken]),
            headers: {
                'Accept': 'application/json, application/octet-stream',
                'Authorization': `Bearer ${challengeHash}`
            },
            responseType: 'arraybuffer',
            onDownloadProgress(progressEvent) {
                if (progressEvent.total === undefined) return;
                onProgressChange(progressEvent.loaded / progressEvent.total)
            },
        })
        const contentType = (response.headers["content-type"] ?? response.headers["Content-Type"])?.toString()
        if (contentType?.startsWith("application/json")) {
            const jsonData = new TextDecoder().decode(response.data)
            const json = JSON.parse(jsonData) as DownloadFileJsonResponse
            return await this.send<ArrayBuffer>({
                method: 'GET',
                url: json.file_url,
                headers: {
                    'Accept': 'application/octet-stream',
                },
                responseType: 'arraybuffer',
                onDownloadProgress(progressEvent) {
                    if (progressEvent.total === undefined) return;
                    onProgressChange(progressEvent.loaded / progressEvent.total)
                },
            })
        } else {
            return response.data
        }
    }

    static async getChallenge(accessToken: string): Promise<GetChallengeResponse> {
        return await this.send({
            method: 'GET',
            url: this.buildEndpoint(['v1', 'files', accessToken, 'challenge']),
            headers: {
                'Accept': 'application/json',
            }
        })
    }

    static async submitChallenge(accessToken: string, challengeHash: string): Promise<SubmitChallengeResponse> {
        const data = {
            challenge: challengeHash,
        }
        return await this.send({
            method: 'POST',
            data,
            url: this.buildEndpoint(['v1', 'files', accessToken, 'challenge']),
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        })
    }

    static async updateExpiry(accessToken: string, updateToken: string, value: number) {
        const data = {
            expiry: value,
        }
        const query = {
            updateToken,
        }
        return await this.send({
            method: 'POST',
            data,
            url: this.buildEndpoint(['v1', 'files', accessToken, 'expiry'], query),
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        })
    }

    /**
     * Generate a download link for a given access token.
     *
     * The password is transfered via the fragment identifier, which is not sent to the server.
     */
    static getDownloadLink(accessToken: string, password: string | null = null): string {
        const baseUrl = `${HOMEPAGE}/${accessToken}`
        const query = password === null ? '' : `#${password}`
        return `${baseUrl}${query}`
    }
}
