import styled from 'styled-components'
import Logo from '../components/Logo'
import Wave from 'react-wavify'

const View = ({ className, children }) => {
    return (
        <div className={className}>
            <Logo />
            <div className="inner">
                {children}
            </div>
            <div className="waves">
                <div className="wave-container">
                    <Wave
                        fill='hsl(0,0%,25%)'
                        paused={false}
                        options={{
                            height: 10,
                            amplitude: 10,
                            speed: 0.25,
                            points: 3,
                        }}
                    />
                </div>
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

    & > .waves {
        position: absolute;
        bottom: 0;
        height: 4rem;
        position: relative;
        width: 100%;

        & > .wave-container {
            width: 100%;
            position: relative;

            & > * {
                top: 0;
                position: absolute;
                width: 100%;
            }
        }
    }
`