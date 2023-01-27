import styled from 'styled-components'
import PropTypes from 'prop-types'

import { Icon } from 'react-feather'
import Spinner from './Spinner'

/**
 * @param {{
 * className: string,
 * label: string,
 * symbol: Icon,
 * isLoading: bool,
 * progress: null | number,
 * }} props
 */
const StatusBubble = ({ className, label, symbol, isLoading = false, progress = null }) => {
    const Symbol = symbol
    const hasProgress = progress !== null
    const progressPercentage = hasProgress ? ((progress || 0) * 100) : 0
    const conicGradient = `conic-gradient(hsl(0,0%,75%) ${progressPercentage}%, transparent ${progressPercentage}%)`

    return (
        <div className={className}>
            <div className="inner" data-loading={isLoading}>
                <div className="symbol" data-loading={isLoading}>
                    <Symbol size={48} />
                </div>
                {!hasProgress && (
                    <div className="spinner" data-loading={isLoading}>
                        <Spinner />
                    </div>
                )}
                {hasProgress && (
                    <div className="progress" data-loading={isLoading} style={{ background: conicGradient }}>
                        <div className="progress__circle" />
                    </div>
                )}
            </div>
            <div className="text" data-loading={isLoading}>
                {label}
            </div>
        </div>
    )
}

StatusBubble.propTypes = {
    className: PropTypes.string,
    isLoading: PropTypes.bool,
}

export default styled(StatusBubble)`
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 2rem;
    max-width: 5rem;

    & > .inner {
        width: 6rem;
        height: 6rem;
        border-radius: 3rem;
        display: flex;
        justify-content: center;
        align-items: center;
        position: relative;
        background-color: hsl(0, 0%,40%);
        overflow: hidden;

        & > .symbol {
            position: absolute;
            display: flex;
            justify-content: center;
            align-items: center;
            color: hsl(0,0%,90%);
            z-index: 1000;

            &[data-loading="true"] {
                animation: pulse 1s infinite;
            }

            @keyframes pulse {
                0% { opacity: 1; }
                50% { opacity: 0.75; }
                100% { opacity: 1; }
            }
        }

        & .spinner {
            position: absolute;
            transform: scale(2.5);
            opacity: 0.33;
            transition: all .2s ease;

            &[data-loading="false"] {
                opacity: 0;
            }
        }

        & .progress {
            position: absolute;
            z-index: 2;
            width: 100%;
            height: 100%;
            border-radius: 3rem;
            opacity: 1;
            transition: all .2s ease;

            &[data-loading="false"] {
                opacity: 0;
            }

            &__circle {
                position: absolute;
                z-index: 3;
                width: 90%;
                height: 90%;
                top: 5%;
                left: 5%;
                border-radius: 3rem;
                background-color: hsl(0, 0%,40%);
            }
        }
    }

    & > .text {
        opacity: 0;
        transition: all .25s ease;

        &[data-loading="true"] {
            opacity: 1;
        }
    }
`