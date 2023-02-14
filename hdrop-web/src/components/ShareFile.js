import { useMemo, useState } from 'react'
import { useRecoilValue } from 'recoil'
import styled from 'styled-components'
import { ArrowRight, Copy, Trash2 } from 'react-feather'

import ApiClient from '../api/ApiClient'
import { accessTokenState, passwordState, updateTokenState } from '../state'
import CopyToClipboardWrapper from './CopyToClipboardWrapper'

const ShareFile = ({ className }) => {
    const accessToken = useRecoilValue(accessTokenState)
    const updateToken = useRecoilValue(updateTokenState)
    const password = useRecoilValue(passwordState)

    /**
     * @param {{
     * includePassword?: boolean
     * }} config
     */
    const getLink = (config) => {
        config = {
            includePassword: false,
            ...config
        }
        return ApiClient.generateLink(accessToken, config.includePassword ? password : null)
    }

    const linkDisplayText = ApiClient.generateLink(accessToken, null).replace(/https?:\/{2}/, '')

    const deleteFile = () => {
        ApiClient.deleteFile(accessToken, updateToken).then(() => {
            window.location.reload()
        })
    }

    return (
        <div className={className}>
            <div className="inner">
                <div className="h-stack">
                    <CopyToClipboardWrapper label="Link" value={getLink()} offset={-0.5}>
                        <div className="link link--primary">
                            <a href={getLink()} target="_blank" rel="noreferrer" onClick={e => e.preventDefault()}>
                                {linkDisplayText}
                            </a>
                            <Copy size={20} />
                        </div>
                    </CopyToClipboardWrapper>
                    {process.env.NODE_ENV === 'development' && (
                        <a className="link link--direct" href={getLink({ includePassword: true })} target="_blank" rel="noreferrer">
                            <ArrowRight size={20} />
                        </a>
                    )}
                </div>
            </div>

            {/* <CopyToClipboardWrapper label="Password" value={password} offset={-0.5}>
                <div className="password" onMouseEnter={() => setShowPassword(true)} onMouseLeave={() => setShowPassword(false)}>
                    <div className="password__label">Password</div>
                    <div className="password__value">
                        {passwordText}
                        <Copy size={14} />
                    </div>
                </div>
            </CopyToClipboardWrapper> */}

            <div className="bottomBar">
                {/* Copy password */}
                <CopyToClipboardWrapper label="Password" value={password} offset={-0.5}>
                    <div className="button">
                        Password
                        <Copy size={14} />
                    </div>
                </CopyToClipboardWrapper>
                {/* Copy link with password */}
                <CopyToClipboardWrapper label="Link with Password" value={getLink({ includePassword: true })} offset={-0.5}>
                    <div className="button">
                        Link with Password
                        <Copy size={14} />
                    </div>
                </CopyToClipboardWrapper>
                <a className="button button--destructive" onClick={() => deleteFile()}>
                    Delete
                    <Trash2 size={14} />
                </a>
            </div>
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
    gap: 2rem;
    width: 100%;

    & .button {
        display: flex;
        align-items: center;
        gap: .5rem;
        background: hsl(0, 0%, 25%);
        border-radius: 0.33rem;
        padding: .33rem .5rem;
        font-size: .9rem;
        overflow: hidden;
        cursor: pointer;
        transition: all .1s ease;

        &:hover {
            background-color: hsl(0, 0%, 30%);
        }

        &--destructive {
            background-color: hsl(0, 55%, 30%);
            color: hsl(0, 0%, 90%);

            &:hover {
                background-color: hsl(0, 65%, 35%);
            }
        }
    }

    & > .bottomBar {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 1rem;
        width: 100%;
        min-width: 100%;
        background: hsl(0, 0%, 15%);
        padding: 1rem;
    }

    & > .inner {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: .5rem;

        & .link {
            display: flex;
            align-items: center;
            gap: 1rem;
            background: hsl(0, 0%, 10%);
            border-radius: 0.5rem;
            padding: .75rem 1rem;
            overflow: hidden;

            a {
                color: inherit;
                text-decoration: none;
                user-select: text;
                text-overflow: ellipsis;
            }

            &--primary {
                flex-grow: 1;
            }

            &--direct {
                color: inherit;
                cursor: pointer;
                transition: all .1s ease;

                &:hover {
                    background-color: hsl(0, 0%, 5%);
                }
            }
        }

        & > .copyWrapper {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: .5rem;
        }
    }

    & .h-stack {
        display: flex;
        justify-content: center;
        gap: .5rem;
        width: 100%;
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
        width: 100%;

        &__label {
            user-select: none;
            cursor: default;
        }

        &__value {
            display: flex;
            align-items: center;
            gap: .25rem;
            background: hsl(0, 0%, 10%);
            border-radius: 0.25rem;
            padding: 0.25rem 0.33rem;
            font-size: .9rem;
            user-select: text;
        }
    }
`