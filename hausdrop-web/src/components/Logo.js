import styled from 'styled-components'
import { Droplet } from 'react-feather'

const Logo = ({ className }) => {
    const handleClick = () => {
        window.location.href = '/'
    }

    return (
        <div className={className} onClick={handleClick}>
            <span><Droplet size={32} /></span>
            <span>hdrop</span>
        </div>
    )
}

export default styled(Logo)`
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
`