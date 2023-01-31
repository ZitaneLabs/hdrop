import styled from 'styled-components'
import View from './View'

const SecurityView = ({ className }) => {
    return (
        <View>
            <div className={className}>
                <article>
                    <h1>Security</h1>
                    <div className="divider" />
                    <h2>TL;DR</h2>
                    <p>
                        hdrop is designed to be highly secure.
                    </p>
                    <h2>How we protect your data</h2>
                    <p>
                        <ul>
                            <li>We exclusively use the modern <a href="https://en.wikipedia.org/wiki/Web_Cryptography_API" target="_blank" rel="noreferrer">Web Crypto API</a></li>
                            <li>Your files and filenames are safely encrypted</li>
                            <li>Encryption and decryption happens on your device</li>
                            <li>We don't store any identifiable file metadata</li>
                        </ul>
                    </p>

                    <h2>Wanna know more?</h2>
                    <p>
                        <ul>
                            <li>Salted PBKDF2 key derivation using 100k rounds</li>
                            <li>File data is encrypted using AES-256-GCM</li>
                        </ul>
                    </p>
                </article>
            </div>
        </View>
    )
}

export default styled(SecurityView)`
    width: 90%;
    overflow: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    color: hsl(0, 0%, 90%);
    background: hsl(0, 0%, 10%);
    border-radius: .5rem;
    padding: 1rem;

    @media screen and (min-width: 700px) {
        width: 80%;
        max-width: 750px;
    }

    h1 {
        width: 100%;
        margin: 0;
    }

    h2 {
        margin: 0;
    }

    article {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        line-height: 1.5rem;
        margin: 1rem 0;

        div.divider {
            width: 100%;
            height: 2px;
            margin: 1rem 0 1.5rem 0;
            background: hsl(0, 0%, 25%);
            border-radius: 1px;
        }

        ul {
            margin: 0;
            padding: 0;
            list-style-position: inside;
        }

        > p {
            margin: 1rem 0 0 1rem;
        }

        h2:not(:first-of-type) {
            margin-top: 1rem;
        }

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