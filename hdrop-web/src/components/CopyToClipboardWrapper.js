import { useEffect, useRef, useState } from 'react'
import styled from 'styled-components'

/**
 * A component wrapper that allows the user to copy a value to their clipboard.
 * 
 * @param {{
 * className: string,
 * label: string,
 * value: string,
 * offset: number,
 * children: React.ReactNode
 * }} props
 */
const CopyToClipboardWrapper = ({ className, label, value, offset = 0, children }) => {
    const timeouts = useRef([])
    const [messageShown, setMessageShown] = useState(false)
    const [lastMouseX, setLastMouseX] = useState(0)
    const [messageX, setMessageX] = useState(0)

    const handleMouseMove = e => {
        setLastMouseX(e.pageX - e.currentTarget.offsetLeft)
        if (!messageShown) {
            setMessageX(lastMouseX)
        }
    }

    const copyToClipboard = () => {
        // Copy to clipboard
        navigator.clipboard.writeText(value).then(() => {
            // Clear timeouts
            while (timeouts.current.length > 0) {
                clearTimeout(timeouts.current.pop())
            }

            // Show success message
            setMessageX(lastMouseX)
            setMessageShown(true)

            // Hide success message after timeout
            timeouts.current.push(setTimeout(() => {
                setMessageShown(false)
            }, 1000))
        })
    }
    
    return (
        <div className={className} onMouseMove={handleMouseMove} onClick={() => copyToClipboard()}>
            <div className="inner">
                {children}
            </div>
            <div className="copy__success" data-show={messageShown} style={{left: `${messageX}px`}}>
                <div className="inner">
                    {`${label} copied`}
                </div>
            </div>
        </div>
    )
}

export default styled(CopyToClipboardWrapper)`
    cursor: pointer;
    position: relative;
    display: flex;

    > .inner {
        display: flex;
        cursor: pointer;
    }

    > .copy__success {
        position: absolute;
        display: flex;
        justify-content: flex-start;
        width: 200px;
        top: ${props => props.offset}rem;
        text-align: center;
        color: hsl(0, 0%, 90%);
        font-size: .75rem;
        margin-top: .5rem;
        pointer-events: none;
        opacity: 0;
        transition: all 0.25s ease;
        transform: translateY(0);

        & > .inner {
            color: hsl(0,0%,90%);
        }

        &[data-show="true"] {
            transform: translateY(-1rem);
            opacity: 1;
        }

        &[data-show="false"] {
            transform: translateY(0);
            opacity: 0;
        }
    }
`