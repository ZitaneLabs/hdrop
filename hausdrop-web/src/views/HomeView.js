import styled from 'styled-components'
import { useState, useEffect } from 'react'
import { useRecoilValue } from 'recoil'
import { Home } from 'react-feather' 
import PropTypes from 'prop-types'

import FileDropZone from '../components/FileDropZone'
import PasswordField from '../components/PasswordField'
import ExpirationPicker from '../components/ExpirationPicker'
import UploadProgress from '../components/UploadProgress'

import { encryptedFileInfoState, fileDataState, passwordState } from '../state'

const HomeView = ({ className }) => {
    const fileData = useRecoilValue(fileDataState)
    const password = useRecoilValue(passwordState)
    const encryptedFileInfo = useRecoilValue(encryptedFileInfoState)
    const [expirySeconds, setExpirySeconds] = useState(null)

    const handleExpirationSubmit = expirySeconds => {
        setExpirySeconds(expirySeconds)
    }

    return (
        <div className={className}>

            <div className="title">
                <span><Home size={32} /></span>
                <span>HausDrop</span>
            </div>

            {/* Stage 1: Select file */}
            <FileDropZone
                hidden={fileData !== null}
            />

            {/* Stage 2: Enter password */}
            <PasswordField
                hidden={fileData === null || password !== null}
            />

            <div className="container">
                <ExpirationPicker
                    onExpirationSubmit={handleExpirationSubmit}
                    active={expirySeconds}
                    defaultValue={24 * 60 * 60}
                    hidden={encryptedFileInfo === null}
                />

                {fileData && password && (
                    <UploadProgress fileData={fileData} password={password} />
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
    justify-content: center;
    align-items: center;
    user-select: none;

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
    }

    & > .container {
        position: absolute;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        pointer-events: none;
        gap: 2rem;
    }
`