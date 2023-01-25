import styled from 'styled-components'
import React, { useState, useMemo, useEffect } from 'react'
import { Loader } from 'react-feather'
import { useRecoilState, useRecoilValue } from 'recoil'

import { CryptoUtil } from '../util'
import { encryptedFileInfoState, fileDataState, fileNameState, passwordState } from '../state'
import Spinner from '../assets/spinner.svg'
import ApiClient from '../api/ApiClient'

const SYM_STEP_ENCRYPT = Symbol('encrypt')
const SYM_STEP_UPLOAD = Symbol('upload')
const SYM_STEP_DONE = Symbol('done')

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

const VisEncrypt = ({ className }) => {
    return (
        <div className={className}>
            <div className="lock">
                <img src={Spinner} alt="Loading" />
            </div>
        </div>
    )
}

const StyledVisEncrypt = styled(VisEncrypt)`
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    color: hsl(0, 0%, 90%);
`

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

    const fileData = useRecoilValue(fileDataState)
    const fileName = useRecoilValue(fileNameState)
    const password = useRecoilValue(passwordState)

    const [encryptedFileInfo, setEncryptedFileInfo] = useRecoilState(encryptedFileInfoState)

    const encryptFile = async () => {
        const derivedKey = await CryptoUtil.deriveKeyFromPassword(password)
        const fileInfo = await CryptoUtil.encryptFile(fileData, fileName, derivedKey)
        setEncryptedFileInfo(fileInfo)
    }

    const uploadFile = async () => {
        console.log(encryptedFileInfo)
        await ApiClient.uploadFile(encryptedFileInfo, progress => {
            console.log(progress)
        })
    }

    const stateMachine = {
        [SYM_STEP_ENCRYPT]: {
            text: 'Encrypting data...',
            action: encryptFile,
            visualization: <StyledVisEncrypt />,
            transition: SYM_STEP_UPLOAD,
            minTime: 2000,
        },
        [SYM_STEP_UPLOAD]: {
            text: 'Uploading data...',
            action: uploadFile,
            visualization: <div />,
            transition: SYM_STEP_DONE,
            minTime: 1000
        },
        [SYM_STEP_DONE]: {
            text: 'Done!',
            action: () => {},
            visualization: <div />,
            transition: null,
            minTime: 0
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
            <div className="error">
                {errorMessage}
            </div>
            {stateMachine[stepSymbol] && (
                <div className="step">
                    <span className="step__text">{stateMachine[stepSymbol].text}</span>
                    <span className="step__visualization">{stateMachine[stepSymbol].visualization}</span>
                </div>
            )}
        </div>
    )
}

export default styled(UploadProgress)`
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    opacity: 0;
    animation: upload-progress-appear .25s ease-in-out forwards;
    color: hsl(0, 0%, 90%);

    & .step {
        display: flex;
        flex-direction: column;
        justify-content: center;
        gap: 1rem;
    }

    @keyframes upload-progress-appear {
        from { opacity: 0; }
        to { opacity: 1; }
    }
`