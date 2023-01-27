import { useState } from 'react'
import { atom, Atom, useRecoilState } from 'recoil'
import styled from 'styled-components'

/**
 * @type {Atom<null | string>}
 */
const activeCopyState = atom({
    key: 'copyToClipboard__activeCopyState',
    default: null
})

/**
 * A component that allows the user to copy a value to their clipboard.
 * 
 * @param {{
 * className: string,
 * label: string,
 * value: string
 * }} props
 */
const CopyToClipboard = ({ className, label, value }) => {
    const [copySuccessful, setCopySuccessful] = useState(false)
    const [activeCopy, setActiveCopy] = useRecoilState(activeCopyState)
    const isCurrentAndSuccessful = copySuccessful && activeCopy === label

    const copyToClipboard = () => {
        navigator.clipboard.writeText(value).then(() => {
            setCopySuccessful(true)
        })
        setActiveCopy(label)
    }
    
    return (
        <div className={className} onClick={() => copyToClipboard()}>
            {isCurrentAndSuccessful ? `${label} copied to clipboard!` : 'Copy to clipboard'}
        </div>
    )
}

export default styled(CopyToClipboard)`
    color: hsl(0, 0%, 90%);
    font-size: .75rem;
    margin-top: .5rem;
    pointer-events: all;
    cursor: pointer;
`