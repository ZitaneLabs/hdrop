import { useState, useMemo, useRef, useEffect } from 'react'
import { useSetRecoilState } from 'recoil'
import styled from 'styled-components'
import zxcvbn from 'zxcvbn'
import PropTypes from 'prop-types'

import { PasswordUtil } from '../util'
import { passwordState } from '../state'

const GaugeColors = {
    0: 'hsl(0, 50%, 50%)',
    1: 'hsl(30, 50%, 50%)',
    2: 'hsl(60, 50%, 50%)',
    3: 'hsl(90, 50%, 50%)',
    4: 'hsl(120, 50%, 50%)',
}

/**
 * A password field that displays the strength of the password.
 * 
 * @param {{
 * className: string,
 * hidden: boolean,
 * }} props
 */
const PasswordField = ({ className, hidden = false }) => {
    const passwordFieldRef = useRef(null)
    const [password, setPassword] = useState('')
    const setPasswordState = useSetRecoilState(passwordState)

    // Focus the password field when it is shown
    useEffect(() => {
        if (hidden) return
        passwordFieldRef.current.focus()
    }, [hidden])

    // Calculate the password strength
    const passwordScore = useMemo(() => {
        return zxcvbn(password).score
    }, [password])

    const handlePasswordSubmitInternal = () => {
        const finalPassword = password.length === 0 ? PasswordUtil.generateSecurePassword() : password
        setPasswordState(finalPassword)
    }

    return (
        <div className={className} data-hidden={hidden}>
            <input
                ref={passwordFieldRef}
                type="password"
                className="inputField"
                value={password}
                onChange={e => setPassword(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && handlePasswordSubmitInternal()}
                placeholder="Password"
                data-gauge-visible={password.length > 0}
            />

            <div
                className="gauge"
                style={{
                    opacity: password.length === 0 ? 0 : 1,
                    width: `${passwordScore / 4.0 * 100}%`,
                    background: GaugeColors[passwordScore]
                }}
            />

            <div className="hint-container">
                <div className="autogenerate" data-hidden={password.length > 0}>
                    Press <kbd>Enter</kbd> to automatically generate a secure password.
                </div>
            </div>
        </div>
    )
}

PasswordField.propTypes = {
    className: PropTypes.string,
    hidden: PropTypes.bool,
}

export default styled(PasswordField)`
    min-width: 300px;
    width: 100%;
    max-width: 750px;
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    opacity: 1;
    pointer-events: all;

    & .inputField {
        -webkit-appearance: none;
        appearance: none;
        border: none;
        outline: none;
        padding: 0 1rem;
        width: 100%;
        height: 5rem;
        background: hsl(0,0%,90%);
        border-radius: 1rem;
        opacity: 0;
        font-size: 2rem;
        transition: all .33s ease-in-out;
        opacity: 1;
        pointer-events: all;
        transform: scale(1);

        &[data-gauge-visible='true'] {
            border-bottom-left-radius: 0;
            border-bottom-right-radius: 0;
        }
    }

    & .gauge {
        align-self: flex-start;
        min-width: 10%;
        max-width: 100%;
        height: .5rem;
        border-radius: 0 0 .25rem .25rem;
        transition: all .33s ease-in-out;
    }

    & .hint-container {
        width: 100%;
        display: flex;
        justify-content: center;
        align-items: center;
        position: relative;
        margin-top: 1rem;
        color: hsl(0,0%,75%);

        & kbd {
            background: hsl(0,0%,10%);
            border-radius: .25rem;
            padding: .25rem .5rem;
        }

        & .autogenerate {
            position: absolute;
            opacity: 1;
            transition: all .25s ease-in-out;
    
            &[data-hidden='true'] {
                opacity: 0;
                transform: translateY(.5rem);
            }
        }
    
        & .complexity {
            margin-top: .5rem;
            position: absolute;
            opacity: 1;
            transition: all .33s ease-in-out;
    
            &[data-hidden='true'] {
                opacity: 0;
            }
        }
    }

    &[data-hidden='true'] {
        opacity: 0;
        pointer-events: none;

        & .inputField {
            transform: scale(0);
        }
    }
`