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

const HomeView = ({ className }) => {
    const fileData = useRecoilValue(fileDataState)
    const password = useRecoilValue(passwordState)
    const encryptedFileInfo = useRecoilValue(encryptedFileInfoState)
    const accessToken = useRecoilValue(accessTokenState)
    const [expirySeconds, setExpirySeconds] = useState(null)
    const fileUploadDone = useRecoilValue(fileFullyUploadedState)

    const handleExpirationSubmit = expirySeconds => {
        setExpirySeconds(expirySeconds)
    }

    return (
        <div className={className}>

            <div className="title">
                <span><Home size={32} /></span>
                <span>HausDrop</span>
            </div>

            <div className="absoluteContainer">
                {/* Stage 1: Select file */}
                <FileDropZone
                    hidden={fileData !== null}
                />

                {/* Stage 2: Enter password */}
                <PasswordField
                    hidden={fileData === null || password !== null}
                />

                {/* Stage 3: Encryption and upload */}
                {fileData && password && (
                    <UploadProgress fileData={fileData} password={password} />
                )}

                {/* Stage 4: Share file */}
                {fileUploadDone && (
                    <div className="container">
                        <ExpirationPicker
                            onExpirationSubmit={handleExpirationSubmit}
                            active={expirySeconds}
                            defaultValue={24 * 60 * 60}
                        />

                        <ShareFile />
                    </div>
                )}
            </div>
        </div>
    )
}

HomeView.propTypes = {
    className: PropTypes.string,
}

export default styled(HomeView)`
    width: 100%;
    height: 100%;
    padding: 1rem;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    user-select: none;

    & > .absoluteContainer {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        position: relative;
        width: 100%;

        & > .container {
            position: absolute;
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

            animation: fadeIn 1s ease-in-out forwards;
            @keyframes fadeIn {
                0% { opacity: 0; }
                100% { opacity: 1; }
            }
        }
    }

    & > .title {
        position: absolute;
        display: flex;
        justify-content: center;
        align-items: center;
        gap: .5rem;
        top: 25%;
        color: hsl(0,0%,90%);
        font-size: 2rem;
        line-height: 2rem;
        cursor: default;
    }
`