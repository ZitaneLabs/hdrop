/** @jest-environment node */
import { ApiClient, Downloader, Uploader } from './'
import { AesGcm, Base64, CHALLENGE_XOR_MASK, CryptoHelper, Sha256 } from '@/crypto'

afterEach(() => jest.restoreAllMocks())

test('uploads ciphertext only and downloads the original file with the correct password', async () => {
    const file = new File(['private file contents 🔒'], 'private filename 🔒.txt')
    const challenge = crypto.getRandomValues(new Uint8Array(32))
    jest.spyOn(CryptoHelper, 'generateChallenge').mockReturnValue(challenge)
    const upload = jest.spyOn(ApiClient, 'uploadFile').mockResolvedValue({ access_token: 'access', update_token: 'update' })
    const onUploadComplete = jest.fn()
    await Uploader.uploadFile(file, jest.fn(), onUploadComplete)

    const data = upload.mock.calls[0][0]
    const multipart = data.toMultipart()
    expect([...multipart.keys()].sort()).toEqual(['challenge_data', 'challenge_hash', 'file_data', 'file_name_data', 'iv', 'salt'])
    expect(await (multipart.get('file_data') as Blob).arrayBuffer()).toEqual(data.fileData)
    expect(multipart.get('file_name_data')).toBe(Base64.encode(new Uint8Array(data.fileNameData)))
    expect(multipart.get('challenge_data')).toBe(Base64.encode(new Uint8Array(data.challengeData)))
    expect(data.fileData.byteLength).toBe(file.size + 16)
    expect(data.fileNameData.byteLength).toBe(new TextEncoder().encode(file.name).byteLength + 16)
    expect(data.challengeData.byteLength).toBe(challenge.byteLength + 16)
    expect(data.challengeHash).toBe(await Sha256.hash(challenge))

    const getChallenge = jest.spyOn(ApiClient, 'getChallenge').mockResolvedValue({
        iv: multipart.get('iv') as string,
        salt: multipart.get('salt') as string,
        challenge: multipart.get('challenge_data') as string,
    })
    const submit = jest.spyOn(ApiClient, 'submitChallenge').mockResolvedValue({ file_name_data: multipart.get('file_name_data') as string })
    const download = jest.spyOn(ApiClient, 'downloadFile').mockResolvedValue(data.fileData)
    const onFileNameObtained = jest.fn()
    const onDownloadComplete = jest.fn()
    const params = {
        accessToken: 'access',
        password: onUploadComplete.mock.calls[0][0].password,
        onProgressChange: jest.fn(),
        onFileNameObtained,
        onDownloadComplete,
    }
    await Downloader.downloadFile(params)
    expect(submit).toHaveBeenCalledWith('access', data.challengeHash)
    expect(download).toHaveBeenCalledWith('access', data.challengeHash, expect.any(Function))
    expect(onFileNameObtained).toHaveBeenCalledWith(file.name)
    expect(onDownloadComplete).toHaveBeenCalledWith({ data: await file.arrayBuffer() })

    submit.mockClear()
    onDownloadComplete.mockClear()
    await expect(Downloader.downloadFile({ ...params, password: 'wrong password' })).rejects.toMatchObject({ name: 'OperationError' })
    expect(submit).not.toHaveBeenCalled()
    expect(onDownloadComplete).not.toHaveBeenCalled()

    // A malicious server substitutes file ciphertext and adjusts metadata so the
    // challenge uses the original file IV. Only the AAD now prevents decryption.
    getChallenge.mockResolvedValue({
        iv: Base64.encode(AesGcm.xorIv(data.iv, CHALLENGE_XOR_MASK)),
        salt: Base64.encode(data.salt),
        challenge: Base64.encode(new Uint8Array(data.fileData)),
    })
    await expect(Downloader.downloadFile(params)).rejects.toMatchObject({ name: 'OperationError' })
    expect(submit).not.toHaveBeenCalled()
    expect(onDownloadComplete).not.toHaveBeenCalled()
})
