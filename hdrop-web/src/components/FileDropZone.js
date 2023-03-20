import styled from 'styled-components'
import { useEffect, useRef, useState } from 'react'
import { FileDrop } from 'react-file-drop'
import { Upload } from 'react-feather'
import { useSetRecoilState } from 'recoil'
import PropTypes from 'prop-types'

import { fileDataState, fileNameState } from '../state'

/**
 * A file drop zone that handles file selection and validation.
 * 
 * @param {{
 * className: string,
 * hidden: boolean,
 * }} props 
 */
const FileDropZone = ({ className, hidden = false }) => {
    const fileInputRef = useRef(null)
    const [errorMessage, setErrorMessage] = useState(null)

    const setFileDataState = useSetRecoilState(fileDataState)
    const setFileNameState = useSetRecoilState(fileNameState)

    const handlePaste = e => {
        const { clipboardData } = e

        // Find files in clipboard
        const viableItems = Array.from(clipboardData.items)
            .filter(item => item.kind === 'file')

        // Validate file count
        if (viableItems.length === 0) {
            setErrorMessage('No file present in clipboard.')
            return
        } else if (viableItems.length > 1) {
            setErrorMessage('Too many files present in clipboard.')
            return
        }

        // Get clipboard item as file
        const file = viableItems[0].getAsFile()

        // Invoke callback
        processFile(file)

        // Clear error messages
        setErrorMessage(null)
    }

    useEffect(() => {
        document.addEventListener('paste', handlePaste)
        return () => {
            document.removeEventListener('paste', handlePaste)
        }
    }, [])

    /**
     * 
     * @param {React.ChangeEvent<HTMLInputElement>} event 
     */
    const handleFileInputChange = event => {
        const { files } = event.target
        handleTargetDrop(files, event)
    }

    const handleTargetClick = () => {
        fileInputRef.current.click()
    }

    const handleTargetDrop = (files, event) => {
        // Check file count for zero
        if (files.length === 0) {
            // TODO: Error, no file selected
            setErrorMessage('No file selected.')
            return
        }

        // Check file count for > 1
        if (files.length > 1) {
            setErrorMessage('You can only upload individual files.')
            return
        }

        // Invoke callback
        processFile(files[0])

        // Clear error messages
        setErrorMessage(null)
    }

    /**
     * @param {File} file
     */
    const processFile = async file => {
        const fileBuffer = await file.arrayBuffer()

        setFileDataState(fileBuffer)
        setFileNameState(file.name)
    }

    return (
        <div className={className} data-hidden={hidden}>
            <input hidden type='file' ref={fileInputRef} onChange={handleFileInputChange} />

            <FileDrop
                onDrop={handleTargetDrop}
                onTargetClick={handleTargetClick}
                className='fileDrop'
                targetClassName='dropTarget'
                draggingOverFrameClassName='dropTarget--drag-frame'
                draggingOverTargetClassName='dropTarget--drag-target'
            >
                <div className="innerDropTarget">
                    <Upload size={48} />
                </div>
            </FileDrop>

            {errorMessage && (
                <div className="errorMessage">{errorMessage}</div>
            )}
        </div>
    )
}

FileDropZone.propTypes = {
    className: PropTypes.string,
    hidden: PropTypes.bool,
}

export default styled(FileDropZone)`
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    opacity: 1;
    pointer-events: all;
    transition: all .25s ease;
    transform: scale(1);

    &[data-hidden='true'] {
        opacity: 0;
        pointer-events: none;
        transform: scale(0);
    }

    & .fileDrop {
        width: 50%;
        height: 50%;
        display: flex;
        justify-content: center;
        align-items: center;

        & .dropTarget {
            width: 10rem;
            height: 10rem;
            background: hsl(0,0%,60%);
            border-radius: 25vmin;
            transition: all .2s ease-in-out;
            transform: scale(0.5);
            box-shadow: 0 0 .25rem 0 hsla(0,0%,0%,.25), 0 0 .75rem 0 hsla(0,0%,0%,.1);
            display: flex;
            justify-content: center;
            align-items: center;

            @media screen and (min-width: 700px) {
                width: 12.5rem;
                height: 12.5rem;
            }

            @media screen and (min-width: 1200px) {
                width: 15rem;
                height: 15rem;
            }

            & .innerDropTarget {
                opacity: 0;
                transition: all .25s ease;
                width: 90%;
                height: 90%;
                border: .25rem dashed hsl(0,0%,25%);
                border-radius: 2.5rem;
                display: flex;
                justify-content: center;
                align-items: center;

                & svg {
                    transition: all .25s ease;
                }
            }

            &:hover:not(&--drag-target):not(&--drag-frame) {
                background: hsl(0,0%,90%);
                transform: scale(0.75);
                border-radius: 2.5rem;
                box-shadow: 0 1rem 0.2rem 0 hsla(0,0%,0%,.05), 0 .33rem .5rem 0 hsla(0,0%,0%,.25);
                cursor: pointer;

                & .innerDropTarget {
                    opacity: 1;
                }
            }

            &.dropTarget--drag-frame {
                background: hsl(0,0%,75%);
                transform: scale(0.75) rotate(45deg);
                border-radius: 2.5rem;

                & .innerDropTarget {
                    opacity: 0;
                    border-radius: 2.5rem;

                    & svg {
                        transform: rotate(-45deg);
                    }
                }
            }

            &.dropTarget--drag-target {
                background: hsl(0,0%,90%);
                transform: scale(1) rotate(90deg);
                border-radius: 1rem;
                box-shadow: 0 0 0.2rem 0 hsla(0,0%,0%,.05), 0 0 .5rem 0 hsla(0,0%,0%,.25);

                & .innerDropTarget {
                    opacity: 1;
                    border-radius: 1rem;

                    & svg {
                        transform: rotate(-90deg);
                    }
                }
            }
        }
    }

    & .errorMessage {
        background: hsl(0,0%,20%);
        padding: 1rem 2rem;
        border-radius: .5rem;
        color: hsl(0,90%,75%);
    }
`