import styled from 'styled-components'
import Logo from '../components/Logo'
import Wave from 'react-wavify'

const View = ({ className, children }) => {
    return (
        <div className={className}>
            <header>
                <Logo />
            </header>
            <main>
                {children}
            </main>
            <footer>
                <div className="links">
                    <a href="/security">Security</a>
                </div>
                <div className="wave-container">
                    <Wave
                        fill='hsl(0,0%,25%)'
                        paused={false}
                        options={{
                            height: 1,
                            amplitude: 10,
                            speed: 0.25,
                            points: 3,
                        }}
                    />
                </div>
            </footer>
        </div>
    )
}

export default styled(View)`
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100vh;
    height: -webkit-fill-available;

    & > header {
        margin-top: 4rem;
        width: 100%;
    }

    & > main {
        width: 100%;
        flex-grow: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
    }

    & > footer {
        width: 100%;
        height: 4rem;
        position: relative;
        margin-top: auto;

        & > .links {
            width: 100%;
            height: 100%;
            position: absolute;
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 100;
            color: hsl(0,0%,90%);
            gap: 1rem;
            text-transform: uppercase;
            margin-top: .33rem;

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
            height: 100%;
            position: relative;

            & > * {
                position: absolute;
                width: 100%;
                height: 100%;
            }
        }
    }
`