import styled from 'styled-components'
import View from './View'

const SecurityView = ({ className }) => {
    return (
        <View>
            <div className={className}>
                <h1>Security</h1>
                <p className="center">
                    hdrop is designed to be very secure.<br />
                    We can't decrypt your files or filenames.
                </p>
                <p>
                    <h2>Cryptopgraphy</h2>
                    <ul>
                        <li>We exclusively use the modern <a href="https://en.wikipedia.org/wiki/Web_Cryptography_API" target="_blank" rel="noreferrer">Web Crypto API</a></li>
                        <li>All encryption happens locally on your device</li>
                        <li>We don't store any unencrypted file metadata</li>
                    </ul>

                    <h2>More Cryptography</h2>
                    <ul>
                        <li>PBKDF2 key derivation using 100k rounds</li>
                        <li>Files are encrypted using AES-256-GCM</li>
                    </ul>
                </p>
            </div>
        </View>
    )
}

export default styled(SecurityView)`
    width: 50%;
    min-width: 450px;
    min-height: 300px;
    height: 75%;
    overflow: auto;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    color: hsl(0, 0%, 90%);
    background: hsl(0, 0%, 10%);
    border-radius: .5rem;

    p {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        line-height: 1.5rem;

        &.center {
            text-align: center;
        }

        a {
            color: hsl(0, 0%, 95%);
            text-decoration: none;
            border-bottom: 1px solid hsl(0, 0%, 50%);
        }
    }
`