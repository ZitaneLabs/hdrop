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
    gap: 2rem;

    opacity: 0;
    animation: appear .25s ease forwards;

    & .file {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        background: hsl(0, 0%, 15%);
        border-radius: .5rem;
        overflow: hidden;
        box-shadow: 0 .25rem .5rem hsla(0, 0%, 0%, 0.25), 0 1rem 2rem hsla(0, 0%, 0%, 0.1);
        border-bottom: 2px solid hsl(0, 0%, 20%);

        &__name {
            color: hsl(0,0%,90%);
            background: hsl(0, 0%, 12.5%);
            border-bottom: 1px solid hsl(0, 0%, 17.5%);
            padding: 1rem 2rem;
            font-size: .8rem;
            max-width: 90%;

            @media screen and (min-width: 700px) {
                font-size: .9rem;
            }

            @media screen and (min-width: 1200px) {
                font-size: 1rem;
            }
        }

        &__preview {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 1rem;
            padding: 2rem;
            border-radius: .25rem;
            width: 100%;
            
            img {
                object-fit: contain;
                width: 100%;
                height: 100%;
                border-radius: .25rem;
            }

            video {
                width: 100%;
                height: 100%;
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