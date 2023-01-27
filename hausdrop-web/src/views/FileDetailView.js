import { useRecoilValue } from 'recoil'
import { Home } from 'react-feather'
import styled from 'styled-components'

import { fileFullyDownloadedState, passwordState } from '../state'
import PasswordField from '../components/PasswordField'
import DownloadProgress from '../components/DownloadProgress'
import ViewFile from '../components/ViewFile'
import Logo from '../components/Logo'
import View from './View'

const FileDetailView = ({ className }) => {
    const password = useRecoilValue(passwordState)
    const isFileDownloaded = useRecoilValue(fileFullyDownloadedState)

    return (
        <View>
            <div className={className}>
                {/* Stage 1: Enter password */}
                {password === null && (
                    <PasswordField />
                )}

                {/* Stage 2: Download and decrypt file */}
                {password !== null && !isFileDownloaded && (
                    <DownloadProgress />
                )}

                {/* Stage 3: Show file */}
                {isFileDownloaded && (
                    <ViewFile />
                )}
            </div>
        </View>
    )
}

export default styled(FileDetailView)`
    width: 100%;
    padding: 1rem;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    user-select: none;
    color: hsl(0,0%,90%);
`