import styled from 'styled-components'
import { Droplet } from 'react-feather'
import { useEffect, useState } from 'react'
import ApiClient from '../api/ApiClient'

const Logo = ({ className }) => {
    const [apiOnline, setApiOnline] = useState(true)

    const checkApiStatus = () => {
        ApiClient.isServerOnline()
            .then(x => setApiOnline(x))
            .catch(() => setApiOnline(false))
    }

    useEffect(() => {
        checkApiStatus()
        const intervalId = setInterval(() => {
            checkApiStatus()
        }, 5000)
        return (() => {
            clearInterval(intervalId)
        })
    })

    const handleClick = () => {
        window.location.href = '/'
    }

    return (
        <div className={className} onClick={handleClick}>
            <div className="inner">
                <span><Droplet size={32} /></span>
                <span>hdrop</span>
            </div>
            {!apiOnline && (
                <div className="status" data-online={apiOnline}>
                    {apiOnline ? 'Online' : 'Server offline'}
                </div>
            )}
        </div>
    )
}

export default styled(Logo)`
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: .5rem;

    & > .inner {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: .5rem;
        color: hsl(0,0%,90%);
        font-size: 2rem;
        line-height: 2rem;
        cursor: default;
        pointer-events: all;
        cursor: pointer;
    }

    & > .status {
        color: hsl(0,0%,90%);
        text-align: center;
        border-radius: 0.25rem;
        padding: 0.2rem 0.5rem;
        text-transform: uppercase;
        font-size: .75rem;

        &[data-online="true"] {
            background: hsl(120, 50%, 20%);
        }

        &[data-online="false"] {
            background: hsl(0, 50%, 20%);
        }
    }
`