import { useState, useEffect } from 'react'
import { useRecoilValue } from 'recoil'
import { Copy } from 'react-feather'
import styled from 'styled-components'
import { decryptedFileInfoState } from '../state'
import { PrismAsync as SyntaxHighlighter } from 'react-syntax-highlighter'
import { oneDark as PrismDarkTheme } from 'react-syntax-highlighter/dist/cjs/styles/prism'

const ViewFile = ({ className }) => {
    const decryptedFile = useRecoilValue(decryptedFileInfoState)
    const [objectUrl, setObjectUrl] = useState(null)

    useEffect(() => {
        setObjectUrl(decryptedFile.objectUrl())
    }, [decryptedFile])

    const isVideo = decryptedFile.isVideo()
    const isImage = decryptedFile.isImage()
    const isAudio = decryptedFile.isAudio()
    const isText = decryptedFile.isText()
    const hasPreview = isVideo || isImage || isAudio || isText

    const copyText = async () => {
        await navigator.clipboard.writeText(decryptedFile.text())
    }

    return (
        <div className={className}>
            <div className="file">
                <div className="file__name">
                    {decryptedFile.name()}
                    {isText && (
                        <div className="copy__text" onClick={() => copyText()}>
                            <Copy size={20} />
                        </div>
                    )}
                </div>
                {hasPreview && objectUrl !== null && (
                    <div className={["file__preview", isAudio ? 'file__preview--100p' : ''].join(' ')}>
                        {isVideo && (
                            <video src={objectUrl} controls />
                        )}
                        {isAudio && (
                            <audio src={objectUrl} controls />
                        )}
                        {isImage && (
                            <img src={objectUrl} alt={decryptedFile.name()} />
                        )}
                        {isText && (
                            <div className="text">
                                <SyntaxHighlighter
                                    language={decryptedFile.guessProgrammingLanguage()}
                                    showLineNumbers
                                    wrapLines
                                    style={PrismDarkTheme}>
                                    {decryptedFile.text()}
                                </SyntaxHighlighter>
                            </div>
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
    height: 100%;

    opacity: 0;
    animation: appear .25s ease forwards;

    & .file {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: flex-start;
        background: hsl(0, 0%, 15%);
        border-radius: .5rem;
        overflow: hidden;
        box-shadow: 0 .25rem .5rem hsla(0, 0%, 0%, 0.25), 0 1rem 2rem hsla(0, 0%, 0%, 0.1);
        /* border-bottom: 2px solid hsl(0, 0%, 20%); */
        height: 100%;

        &__name {
            position: relative;
            display: flex;
            justify-content: center;
            align-items: center;
            color: hsl(0,0%,90%);
            background: hsl(0, 0%, 12.5%);
            border-bottom: 1px solid hsl(0, 0%, 17.5%);
            padding: 1rem 2rem;
            font-size: .8rem;
            width: 100%;
            overflow: hidden;
            text-overflow: ellipsis;
            text-align: center;

            .copy__text {
                position: absolute;
                right: .25rem;
                display: flex;
                justify-content: center;
                align-items: center;
                width: 28px;
                height: 28px;
                border-radius: .33rem;
                cursor: pointer;

                &:hover {
                    background: hsl(0,0%,25%);
                }
            }

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

            &--100p {
                width: 100%;
            }

            img {
                object-fit: scale-down;
                max-width: 100%;
                max-height: 100%;
            }

            audio {
                width: 100%;
            }

            video {
                max-width: 100%;
                max-height: 100%;
            }

            div.text {
                position: relative;
                max-width: 100vw;
                max-height: 100%;
                overflow: auto;

                pre {
                    margin: 0 !important;
                    padding: 0 !important;
                    user-select: all;
                    pointer-events: all;
                }
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