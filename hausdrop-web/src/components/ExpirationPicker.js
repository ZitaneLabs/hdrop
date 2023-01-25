import { useEffect } from 'react'
import styled from 'styled-components'

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
 * onExpirationSubmit: (expirationSeconds: number) => void,
 * active?: number,
 * hidden: boolean
 * }} props
 */
const ExpirationPicker = ({ className, onExpirationSubmit, active = null, defaultValue = null, hidden = false }) => {
    useEffect(() => {
        if (hidden) return
        if (defaultValue) {
            onExpirationSubmit(defaultValue)
        }
    }, [hidden])

    return (
        <div className={className} data-hidden={hidden}>
            <div className="title">
                Expiration Time
            </div>
            <div className="expiry__container">
                {ExpirationIncrements.map(({ label, seconds }) => (
                    <div
                        key={seconds}
                        className="expiry"
                        data-active={active === seconds}
                        onClick={() => onExpirationSubmit(seconds)}>
                        {label}
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
    gap: 1rem;
    transition: all .33s ease-in-out;
    opacity: 1;
    pointer-events: all;
    transform: scale(1);

    & .title {
        color: hsl(0,0%,90%);
        font-size: 1.25rem;
    }

    & .expiry__container {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 1.5rem;
    }

    & .expiry {
        display: flex;
        justify-content: center;
        align-items: center;
        width: 3.5rem;
        height: 3.5rem;
        font-size: 1.25rem;
        background: hsl(0,0%,90%);
        border-radius: 2.5rem;
        transition: all .2s ease-in-out;
        cursor: pointer;

        &[data-active='false'] {
            transform: scale(0.9);
        }

        &[data-active='true'] {
            border: .25rem solid hsl(0,0%,75%);
            box-shadow: 0 0 .5rem 0 hsla(0,0%,100%,.25);
            transform: scale(1.1);
            background: hsl(0,0%,45%);
            color: hsl(0,0%,90%);
        }

        &:hover {
            transform: scale(1.2) rotate(9deg);
            box-shadow: 0 .25rem .5rem 0 hsla(0,0%,0%,.25);
            font-size: 1.33rem;
            border-radius: 2rem;
        }
    }

    &[data-hidden='true'] {
        opacity: 0;
        pointer-events: none;
        transform: scale(0);
    }
`