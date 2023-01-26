import { useState, useEffect } from 'react'
import { useRecoilValue } from 'recoil'
import styled from 'styled-components'

import ApiClient from '../api/ApiClient'
import { accessTokenState, passwordAutogeneratedState, passwordState } from '../state'
import CopyToClipboard from './CopyToClipboard'

const ShareFile = ({ className }) => {
    const accessToken = useRecoilValue(accessTokenState)
    const link = ApiClient.generateLink(accessToken)
    const isPasswordAutogenerated = useRecoilValue(passwordAutogeneratedState)
    const password = useRecoilValue(passwordState)

    return (
        <div className={className}>
            <div className="link">
                <a href={link} target="_blank">{link.replace(/https?:\/{2}/, '')}</a>
            </div>
            <CopyToClipboard label="Link" value={link} />
            {isPasswordAutogenerated && (
                <>
                    <div className="password">
                        <div className="password__label">Password</div>
                        <div className="password__value">{password}</div>
                    </div>
                    <CopyToClipboard label="Password" value={password} />
                </>
            )}
        </div>
    )
}

export default styled(ShareFile)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: hsl(0, 0%, 90%);
    pointer-events: all;

    & > .link {
        background: hsl(0, 0%, 10%);
        border-radius: 0.5rem;
        padding: 1rem;

        a {
            color: inherit;
            text-decoration: none;
            user-select: text;
        }
    }

    & > .copy {
        color: hsl(0, 0%, 90%);
        font-size: .75rem;
        cursor: pointer;
        margin-top: .5rem;
    }

    & .password {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: .5rem;
        font-size: .9rem;
        margin-top: 2rem;

        &__label {
        }

        &__value {
            background: hsl(0, 0%, 10%);
            border-radius: 0.25rem;
            padding: 0.25rem 0.33rem;
            font-size: .9rem;
            user-select: text;
        }
    }
`