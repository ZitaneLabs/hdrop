import styled from 'styled-components'
import Logo from '../components/Logo'

const View = ({ className, children }) => {
    return (
        <div className={className}>
            <Logo />
            <div className="inner">
                {children}
            </div>
        </div>
    )
}

export default styled(View)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;

    & > .inner {
        width: 100%;
        height: 90%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }
`