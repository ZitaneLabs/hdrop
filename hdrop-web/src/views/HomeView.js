import styled from 'styled-components'
import { useState, useEffect } from 'react'
import { useRecoilValue } from 'recoil'
import { Home } from 'react-feather' 
import PropTypes from 'prop-types'

import FileDropZone from '../components/FileDropZone'
import PasswordField from '../components/PasswordField'
import ExpirationPicker from '../components/ExpirationPicker'
import UploadProgress from '../components/UploadProgress'

import { accessTokenState, encryptedFileInfoState, fileDataState, fileFullyUploadedState, passwordState } from '../state'
import ShareFile from '../components/ShareFile'
import Logo from '../components/Logo'
import View from './View'

const HomeView = ({ className }) => {
    const fileData = useRecoilValue(fileDataState)
    const password = useRecoilValue(passwordState)
    const encryptedFileInfo = useRecoilValue(encryptedFileInfoState)
    const accessToken = useRecoilValue(accessTokenState)
    const fileUploadDone = useRecoilValue(fileFullyUploadedState)

    return (
        <View>
            <div className={className}>
                {/* Stage 1: Select file */}
                {fileData === null && (
                    <FileDropZone
                        hidden={fileData !== null}
                    />
                )}

                {/* Stage 2: Enter password */}
                {fileData !== null && password === null && (
                    <PasswordField
                        showComplexityScore
                        allowPasswordGeneration
                    />
                )}

                {/* Stage 3: Encryption and upload */}
                {fileData && password && (
                    <UploadProgress fileData={fileData} password={password} />
                )}

                {/* Stage 4: Share file */}
                {fileUploadDone && (
                    <div className="container">
                        <ExpirationPicker
                            defaultValue={24 * 60 * 60}
                        />

                        <ShareFile />
                    </div>
                )}
            </div>
        </View>
    )
}

HomeView.propTypes = {
    className: PropTypes.string,
}

export default styled(HomeView)`
    width: 100%;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    user-select: none;

    opacity: 0;
    animation: appear .25s ease forwards;

    & > .container {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        gap: 2rem;
        background: hsla(0,0%,0%,.15);
        border-radius: 1rem;
        padding: 2.5rem;
        min-width: 300px;
        max-width: 700px;
        width: 100%;

        opacity: 0;
        animation: appear .25s ease forwards;
    }
`