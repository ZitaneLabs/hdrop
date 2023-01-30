import styled from 'styled-components'
import React, { useState, useEffect } from 'react'
import { useRecoilState, useRecoilValue, useSetRecoilState } from 'recoil'

import { CryptoUtil } from '../util'

import {
    encryptedFileInfoState,
    fileDataState,
    fileNameState,
    passwordState,
    accessTokenState,
    updateTokenState,
    fileFullyUploadedState
} from '../state'

import ApiClient from '../api/ApiClient'

import UploadProgressBar from './UploadProgressBar'
import Spinner from './Spinner'
import StatusBubbleRow from './StatusBubbleRow'
import StatusBubble from './StatusBubble'
import { Check, Cpu, Upload } from 'react-feather'

const SYM_STEP_ENCRYPT = 'encrypt'
const SYM_STEP_UPLOAD = 'upload'
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
const UploadProgress = ({ className }) => {
    const [stepSymbol, setStepSymbol] = useState(SYM_STEP_ENCRYPT)
    const [errorMessage, setErrorMessage] = useState(null)
    const [uploadProgress, setUploadProgress] = useState(0)

    const fileData = useRecoilValue(fileDataState)
    const fileName = useRecoilValue(fileNameState)
    const password = useRecoilValue(passwordState)

    const setAccessToken = useSetRecoilState(accessTokenState)
    const setUpdateToken = useSetRecoilState(updateTokenState)
    const setFileFullyUploaded = useSetRecoilState(fileFullyUploadedState)

    const [encryptedFileInfo, setEncryptedFileInfo] = useRecoilState(encryptedFileInfoState)

    const encryptFile = async () => {
        const derivedKey = await CryptoUtil.deriveKeyFromPassword(password)
        const fileInfo = await CryptoUtil.encryptFile(fileData, fileName, derivedKey)
        setEncryptedFileInfo(fileInfo)
    }

    const uploadFile = async () => {
        try {
            const data = await ApiClient.uploadFile(encryptedFileInfo, progress => {
                setUploadProgress(progress)
            })
            setAccessToken(data.access_token)
            setUpdateToken(data.update_token)
        } catch (err) {
            console.log(err.message)
            setErrorMessage(err.message)
            return false
        }
    }

    const stateMachine = {
        [SYM_STEP_ENCRYPT]: {
            action: encryptFile,
            visualization: <Spinner />,
            transition: SYM_STEP_UPLOAD,
            minTime: 1000,
        },
        [SYM_STEP_UPLOAD]: {
            action: uploadFile,
            visualization: <UploadProgressBar progress={uploadProgress} />,
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
                setFileFullyUploaded(true)
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
                            symbol={Cpu}
                            label="Encrypting"
                            isLoading={stepSymbol === SYM_STEP_ENCRYPT}
                        />
                        <StatusBubble
                            symbol={Upload}
                            label="Uploading"
                            progress={uploadProgress}
                            isLoading={stepSymbol === SYM_STEP_UPLOAD}
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

export default styled(UploadProgress)`
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