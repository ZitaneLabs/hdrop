export type UploadFileResponse = {
    access_token: string
    update_token: string
}

export type GetChallengeResponse = {
    iv: string
    salt: string
    challenge: string
}

export type SubmitChallengeResponse = {
    file_name_data: string
}

export type DownloadFileJsonResponse = {
    file_url: string
}

export type DownloadFileBinaryResponse = Uint8Array
