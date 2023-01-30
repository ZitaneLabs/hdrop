import { useEffect } from 'react'
import { useRecoilState, useRecoilValue } from 'recoil'
import styled from 'styled-components'
import ApiClient from '../api/ApiClient'
import { accessTokenState, expirySecondsState, updateTokenState } from '../state'

const ExpirationIncrements = [
    { label: '15m', seconds: 15 * 60, },
    { label: '1h', seconds: 60 * 60, },
    { label: '6h', seconds: 6 * 60 * 60, },
    { label: '12h', seconds: 12 * 60 * 60, },
    { label: '24h', seconds: 24 * 60 * 60, },
]

/**
 * 
 * @param {{
 * className: string,
 * defaultValue: number,
 * }} props
 */
const ExpirationPicker = ({ className, defaultValue }) => {
    const accessToken = useRecoilValue(accessTokenState)
    const updateToken = useRecoilValue(updateTokenState)
    const [expirySeconds, setExpirySeconds] = useRecoilState(expirySecondsState)

    const activeIncrement = expirySeconds === null ? defaultValue : expirySeconds

    const handleSubmit = async seconds => {
        await ApiClient.updateFileExpiration(accessToken, updateToken, seconds)
        setExpirySeconds(seconds)
    }

    return (
        <div className={className}>
            <div className="title">
                Expiration Time
            </div>
            <div className="expiry__container">
                {ExpirationIncrements.map(({ label, seconds }) => (
                    <div
                        key={seconds}
                        className="expiry"
                        data-active={seconds === activeIncrement}
                        onClick={() => handleSubmit(seconds)}>
                        <span>{label}</span>
                        <div className="bottom-indicator" />
                    </div>
                ))}
            </div>
        </div>
    )
}

export default styled(ExpirationPicker)`
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: .5rem;
    transition: all .33s ease-in-out;
    opacity: 1;
    pointer-events: all;
    transform: scale(1);

    & .title {
        color: hsl(0,0%,90%);
        cursor: default;
    }

    & .expiry__container {
        display: flex;
        justify-content: center;
        align-items: center;
        border-radius: .5rem;
        box-shadow: 0 .5rem .75rem hsla(0,0%,0%,.25);
    }

    & .expiry {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        position: relative;
        padding: .5rem 1rem;
        background: hsl(0,0%,25%);
        color: hsl(0,0%,80%);
        transition: all .2s ease;
        overflow: hidden;
        cursor: pointer;

        & .bottom-indicator {
            position: absolute;
            bottom: 0;
            width: 100%;
            height: .1rem;
            background: hsl(0,0%,30%);
        }
        
        &:first-child {
            border-top-left-radius: .5rem;
            border-bottom-left-radius: .5rem;
        }

        &:last-child {
            border-top-right-radius: .5rem;
            border-bottom-right-radius: .5rem;
        }

        &:hover {
            background: hsl(0,0%,30%);
            color: hsl(0,0%,90%);

            & .bottom-indicator {
                background: hsl(0,0%,40%);
            }
        }

        &[data-active='true'] {
            background: hsl(0,0%,40%);
            color: hsl(0,0%,90%);

            & .bottom-indicator {
                background: hsl(0,0%,50%);
            }
        }
    }
`