import { useRecoilValue } from 'recoil'
import { Home } from 'react-feather'
import styled from 'styled-components'

import { fileFullyDownloadedState, passwordState } from '../state'
import PasswordField from '../components/PasswordField'
import DownloadProgress from '../components/DownloadProgress'
import ViewFile from '../components/ViewFile'

const FileDetailView = ({ className }) => {
    const password = useRecoilValue(passwordState)
    const isFileDownloaded = useRecoilValue(fileFullyDownloadedState)

    return (
        <div className={className}>
            <div className="title">
                <span><Home size={32} /></span>
                <span>HausDrop</span>
            </div>

            <div className="absoluteContainer">
                {/* Stage 1: Enter password */}
                <PasswordField hidden={password !== null} />

                {/* Stage 2: Download and decrypt file */}
                {password !== null && !isFileDownloaded && (
                    <DownloadProgress />
                )}

                {/* Stage 3: Show file */}
                {isFileDownloaded && (
                    <ViewFile />
                )}
            </div>
        </div>
    )
}

export default styled(FileDetailView)`
    width: 100%;
    height: 100%;
    padding: 1rem;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    user-select: none;
    color: hsl(0,0%,90%);

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

    & > .absoluteContainer {
        display: flex;
        flex-direction: column;
        justify-content: flex-start;
        align-items: center;
        position: relative;
        width: 100%;
    }
`