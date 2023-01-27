import styled from 'styled-components'
import React, { useState, useEffect, useMemo, useCallback } from 'react'
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil'
import { Check, Cpu, Download } from 'react-feather'

import ApiClient from '../api/ApiClient'
import { EncryptedFileInfo } from '../util'
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

    const [stepSymbol, setStepSymbol] = useState(SYM_STEP_DOWNLOAD)
    const [errorMessage, setErrorMessage] = useState(null)
    const [downloadProgress, setDownloadProgress] = useState(0)

    const password = useRecoilValue(passwordState)
    const setFileFullyDownloaded = useSetRecoilState(fileFullyDownloadedState)

    const [downloadedFileData, setDownloadedFileData] = useRecoilState(downloadedFileInfoState)
    const [decryptedFileInfo, setDecryptedFileInfo] = useRecoilState(decryptedFileInfoState)

    const downloadFile = async () => {
        ApiClient.getFile(accessToken, progress => {
            setDownloadProgress(progress)
        }).then(data => {
            setDownloadedFileData(data)
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
        [SYM_STEP_DOWNLOAD]: {
            action: downloadFile,
            visualization: <UploadProgressBar progress={downloadProgress} />,
            transition: SYM_STEP_DECRYPT,
            minTime: 1000,
        },
        [SYM_STEP_DECRYPT]: {
            action: decryptFile,
            visualization: <Spinner />,
            transition: SYM_STEP_DONE,
            minTime: 1000,
        },
        [SYM_STEP_DONE]: {
            action: () => {},
            visualization: <div />,
            transition: SYM_STEP_DONE2,
            minTime: 500,
        },
        [SYM_STEP_DONE2]: {
            action: () => {
                setFileFullyDownloaded(true)
            },
            visualization: <div />,
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
                await doTimedStep(action, minTime)
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