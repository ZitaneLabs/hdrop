import styled from 'styled-components'
import { Droplet } from 'react-feather'

const Logo = ({ className }) => {
    const handleClick = () => {
        window.location.href = '/'
    }

    return (
        <div className={className} onClick={handleClick}>
            <div className="inner">
                <span><Droplet size={32} /></span>
                <span>hdrop</span>
            </div>
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