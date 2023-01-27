import styled from 'styled-components'
import { Home } from 'react-feather'
import { useNavigate } from 'react-router-dom'

const Logo = ({ className }) => {
    const navigate = useNavigate()

    const handleClick = () => {
        window.location.href = '/'
    }

    return (
        <div className={className} onClick={handleClick}>
            <span><Home size={32} /></span>
            <span>HausDrop</span>
        </div>
    )
}

export default styled(Logo)`
    position: absolute;
    top: 4rem;
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