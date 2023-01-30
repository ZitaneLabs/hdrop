import { useRecoilState, useRecoilValue } from 'recoil'
import styled from 'styled-components'

import { fileFullyDownloadedState, passwordState } from '../state'
import PasswordField from '../components/PasswordField'
import DownloadProgress from '../components/DownloadProgress'
import ViewFile from '../components/ViewFile'
import Logo from '../components/Logo'
import View from './View'
import { useEffect, useState } from 'react'
import { useLocation } from 'react-router-dom'

const FileDetailView = ({ className }) => {
    const [isLoading, setIsLoading] = useState(true)
    const [password, setPassword] = useRecoilState(passwordState)
    const isFileDownloaded = useRecoilValue(fileFullyDownloadedState)
    const { hash } = useLocation()

    useEffect(() => {
        if (hash && hash.length > 0) {
            console.log(hash.slice(1))
            setPassword(hash.slice(1))
        }
        setIsLoading(false)
    }, [])

    return (
        <View>
            {!isLoading && (
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
            )}
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