import styled from 'styled-components'

import SpinnerSvg from '../assets/spinner.svg'

const Spinner = ({ className }) => {
    return (
        <div className={className}>
            <img src={SpinnerSvg} alt="Loading" />
        </div>
    )
}

export default styled(Spinner)`
    display: flex;
    justify-content: center;
    align-items: center;
    color: hsl(0, 0%, 90%);
`