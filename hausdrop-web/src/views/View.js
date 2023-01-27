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
            <div className="footer">
                <div className="links">
                    <a href="/security">Security</a>
                </div>
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

    & > .footer {
        position: absolute;
        bottom: 0;
        height: 4rem;
        position: relative;
        width: 100%;

        & > .links {
            width: 100%;
            position: absolute;
            bottom: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 100;
            color: hsl(0,0%,90%);
            gap: 1rem;
            text-transform: uppercase;

            a {
                color: inherit;
                text-decoration: none;
                border-bottom: 1px solid hsl(0,0%,45%);
                font-size: 0.8rem;

                &:hover {
                    border-bottom: 1px solid hsl(0,0%,75%);
                }
            }
        }

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