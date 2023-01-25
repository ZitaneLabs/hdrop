import { atom, RecoilState } from 'recoil'

import { EncryptedFileInfo } from '../util'

/**
 * @type {RecoilState<Uint8Array | null>}
 */
export const fileDataState = atom({
    key: 'fileData',
    default: null,
})

/**
 * @type {RecoilState<string | null>}
 */
export const fileNameState = atom({
    key: 'fileName',
    default: null,
})

/**
 * @type {RecoilState<string | null>}
 */
export const passwordState = atom({
    key: 'password',
    default: null,
})

/**
 * @type {RecoilState<number | null>}
 */
export const expirySecondsState = atom({
    key: 'expirySeconds',
    default: null,
})

/**
 * @type {RecoilState<EncryptedFileInfo | null>}
 */
export const encryptedFileInfoState = atom({
    key: 'encryptedFileInfo',
    default: null,
})