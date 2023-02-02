import { useState, useEffect } from 'react'
import { useRecoilValue } from 'recoil'
import styled from 'styled-components'
import { decryptedFileInfoState } from '../state'

const ViewFile = ({ className }) => {
    const decryptedFile = useRecoilValue(decryptedFileInfoState)
    const [objectUrl, setObjectUrl] = useState(null)

    useEffect(() => {
        setObjectUrl(decryptedFile.objectUrl())
    }, [decryptedFile])

    const isVideo = decryptedFile.isVideo()
    const isImage = decryptedFile.isImage()
    const isAudio = decryptedFile.isAudio()

    const hasPreview = isVideo || isImage || isAudio

    return (
        <div className={className}>
            <div className="file">
                <div className="file__name">{decryptedFile.name()}</div>
                {hasPreview && objectUrl !== null && (
                    <div className="file__preview">
                        {isVideo && (
                            <video src={objectUrl} controls />
                        )}
                        {isAudio && (
                            <audio src={objectUrl} controls />
                        )}
                        {isImage && (
                            <img src={objectUrl} alt={decryptedFile.name()} />
                        )}
                    </div>
                )}
            </div>
            {objectUrl !== null && (
                <div className="download">
                    <a href={objectUrl} download={decryptedFile.name()}>Download file</a>
                </div>
            )}
        </div>
    )
}

export default styled(ViewFile)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: hsl(0, 0%, 90%);
    pointer-events: all;
    overflow: hidden;
    gap: 2rem;
    height: 100%;
    height: -webkit-fill-available;

    opacity: 0;
    animation: appear .25s ease forwards;

    & .file {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: flex-start;
        background: hsl(0, 0%, 15%);
        border-radius: .5rem;
        overflow: auto;
        box-shadow: 0 .25rem .5rem hsla(0, 0%, 0%, 0.25), 0 1rem 2rem hsla(0, 0%, 0%, 0.1);
        border-bottom: 2px solid hsl(0, 0%, 20%);
        gap: 2rem;
        height: 100%;

        &__name {
            color: hsl(0,0%,90%);
            background: hsl(0, 0%, 12.5%);
            border-bottom: 1px solid hsl(0, 0%, 17.5%);
            padding: 1rem 2rem;
            font-size: .8rem;
            max-width: 90%;
            overflow: hidden;
            text-overflow: ellipsis;

            @media screen and (min-width: 700px) {
                font-size: .9rem;
            }

            @media screen and (min-width: 1200px) {
                font-size: 1rem;
            }
        }

        &__preview {
            display: flex;
            overflow: hidden;
            flex-shrink: 1;

            img {
                object-fit: scale-down;
                border-radius: .25rem;
                width: 100%;
                max-height: 100%;
            }

            video {
                max-width: 100%;
                max-height: 100%;
                border-radius: .25rem;
            }
        }
    }

    & .download {
        background: hsl(0, 0%, 10%);
        padding: 1rem 2rem;
        border-radius: .25rem;
        cursor: pointer;
        width: 75%;
        text-align: center;
        box-shadow: 0 .25rem .5rem hsla(0, 0%, 0%, 0.25), 0 1rem 2rem hsla(0, 0%, 0%, 0.1);
        border-bottom: 2px solid hsl(0, 0%, 20%);

        &:hover {
            background: hsl(0, 0%, 12.5%);
        }

        & > a {
            color: hsl(0, 0%, 90%);
            text-decoration: none;
        }
    }
`