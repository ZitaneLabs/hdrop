import styled from 'styled-components'
import React, { useState, useEffect, useMemo, useCallback } from 'react'
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil'
import { Check, Cpu, Download, Hash, Key } from 'react-feather'

import ApiClient from '../api/ApiClient'
import { Base64Util, CryptoUtil, EncryptedFileInfo } from '../util'
import {
    passwordState,
    fileFullyDownloadedState,
    downloadedFileInfoState,
    decryptedFileInfoState
} from '../state'


import UploadProgressBar from './UploadProgressBar'
import Spinner from './Spinner'
import StatusBubbleRow from './StatusBubbleRow'
import StatusBubble from './StatusBubble'
import { useParams } from 'react-router-dom'

const SYM_STEP_CHALLENGE = 'challenge'
const SYM_STEP_DOWNLOAD = 'download'
const SYM_STEP_DECRYPT = 'decrypt'
const SYM_STEP_DONE = 'done'
const SYM_STEP_DONE2 = 'done2'

/**
 * 
 * @param {<T>() => Promise<T>} stepPromise 
 * @param {number} minTime 
 * @returns 
 */
const doTimedStep = async (stepPromise, minTime) => {
    const startTime = Date.now()
    const result = await stepPromise()
    const endTime = Date.now()
    const timeDiff = endTime - startTime
    if (timeDiff < minTime) {
        await new Promise(resolve => setTimeout(resolve, minTime - timeDiff))
    }
    return result
}

/**
 * 
 * @param {{
 * className: string,
 * }} props 
 * @returns 
 */
const DownloadProgress = ({ className }) => {
    const { accessToken } = useParams()

    const [stepSymbol, setStepSymbol] = useState(SYM_STEP_CHALLENGE)
    const [errorMessage, setErrorMessage] = useState(null)
    const [downloadProgress, setDownloadProgress] = useState(0)

    const password = useRecoilValue(passwordState)
    const setFileFullyDownloaded = useSetRecoilState(fileFullyDownloadedState)

    const [downloadedFileData, setDownloadedFileData] = useRecoilState(downloadedFileInfoState)
    const [decryptedFileInfo, setDecryptedFileInfo] = useRecoilState(decryptedFileInfoState)

    const solveChallenge = async () => {
        // Request challenge data
        const challengeResponse = await ApiClient.getChallenge(accessToken);

        // Decode challenge data
        const challengeData = Base64Util.decode(challengeResponse.challenge)
        const saltData = Base64Util.decode(challengeResponse.salt)
        const ivData = Base64Util.decode(challengeResponse.iv)

        // Derive key
        const recoveredKey = await CryptoUtil.recoverKeyFromPassword(password, saltData, ivData)

        // Solve challenge
        let challengeSolutionData;
        try {
            challengeSolutionData = await CryptoUtil.decryptString(challengeData, recoveredKey)
        } catch {
            // Invalid challenge or network error
            setErrorMessage('Unable to solve cryptographic challenge. Please check your password.')

            // Prevent state transition
            return false
        }

        const challengeSolution = await CryptoUtil.hashStringHex(challengeSolutionData)

        try {
            // Submit challenge
            await ApiClient.submitChallenge(accessToken, challengeSolution)
        } catch {
            // Invalid challenge or network error
            setErrorMessage('Unable to solve cryptographic challenge. Please check your password.')

            // Prevent state transition
            return false
        }
    }

    const downloadFile = async () => {
        ApiClient.getFile(accessToken, progress => {
            setDownloadProgress(progress)
        }).then(data => {

            // Check if file is provided as URL
            if (data.file_url !== null) {
                // Download file
                ApiClient.directDownloadFile(data.file_url).then(fileContents => {
                    // Set file data in original data
                    data.file_data = fileContents
                    setDownloadedFileData(data)
                }).catch(({ reason }) => {
                    console.log(reason)
                    setErrorMessage(reason)
                })
                return
            } else {
                setDownloadedFileData(data)
            }
        }).catch(({ reason }) => {
            console.log(reason)
            setErrorMessage(reason)
        })
    }

    const decryptFile = async () => {
        const encryptedFileInfo = await EncryptedFileInfo.fromEncryptedFileData(
            downloadedFileData,
            password
        )
        const fileInfo = await encryptedFileInfo.decrypt()
        setDecryptedFileInfo(fileInfo)
    }

    const stateMachine = {
        [SYM_STEP_CHALLENGE]: {
            action: solveChallenge,
            visualization: <Spinner />,
            transition: SYM_STEP_DOWNLOAD,
            minTime: 250,
        },
        [SYM_STEP_DOWNLOAD]: {
            action: downloadFile,
            visualization: <UploadProgressBar progress={downloadProgress} />,
            transition: SYM_STEP_DECRYPT,
            minTime: 250,
        },
        [SYM_STEP_DECRYPT]: {
            action: decryptFile,
            visualization: <Spinner />,
            transition: SYM_STEP_DONE,
            minTime: 500,
        },
        [SYM_STEP_DONE]: {
            action: () => {},
            visualization: null,
            transition: SYM_STEP_DONE2,
            minTime: 500,
        },
        [SYM_STEP_DONE2]: {
            action: () => {
                setFileFullyDownloaded(true)
            },
            visualization: null,
            transition: null,
        }
    }

    useEffect(() => {
        (async () => {
            const symbol = stepSymbol
            const state = stateMachine[symbol]
            if (!state) return
    
            const { action, transition, minTime } = state
            try {
                if (await doTimedStep(action, minTime) === false) {
                    return
                }
                if (transition) {
                    setStepSymbol(transition)
                }
            }
            catch (err) {
                setErrorMessage(err.message)
            }
        })()
    }, [stepSymbol])

    return (
        <div className={className}>
            {stepSymbol !== SYM_STEP_DONE2 && (
                <div className="statusBubbleContainer">
                    <StatusBubbleRow>
                        <StatusBubble
                            symbol={Key}
                            label="Validating"
                            isLoading={stepSymbol === SYM_STEP_CHALLENGE}
                        />
                        <StatusBubble
                            symbol={Download}
                            label="Downloading"
                            progress={downloadProgress}
                            isLoading={stepSymbol === SYM_STEP_DOWNLOAD}
                        />
                        <StatusBubble
                            symbol={Cpu}
                            label="Decrypting"
                            isLoading={stepSymbol === SYM_STEP_DECRYPT}
                        />
                        <StatusBubble
                            symbol={Check}
                            label="Done"
                            isLoading={stepSymbol === SYM_STEP_DONE}
                        />
                    </StatusBubbleRow>
                </div>
            )}
            <div className="error" data-hidden={errorMessage === null}>
                {errorMessage}
            </div>
        </div>
    )
}

export default styled(DownloadProgress)`
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    color: hsl(0, 0%, 90%);
    gap: 2rem;

    opacity: 0;
    animation: appear .25s ease forwards;

    & > .statusBubbleContainer {
        transform: all .25s ease;
        pointer-events: none;

        opacity: 0;
        animation: appear .25s ease forwards;
    }

    & > .error {
        display: flex;
        justify-content: flex-start;
        align-items: center;
        padding: .5rem 0.75rem;
        width: 100%;
        min-height: calc(1.5rem + (0.5rem * 2) + 4px);
        line-height: 1.5rem;
        background-color: hsl(0, 25%, 20%);
        color: hsl(0, 100%, 85%);
        border-radius: .5rem;
        border: 2px solid hsl(0, 33%, 30%);
        transition: all .25s ease;

        &[data-hidden="true"] {
            opacity: 0;
            min-height: 0;
            padding: 0;
        }
    }
`