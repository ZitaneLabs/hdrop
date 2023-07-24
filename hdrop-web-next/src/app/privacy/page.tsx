export default function PrivacyPage() {
    return (
        <main className="flex flex-col items-center justify-center h-full overflow-auto">
            <p className="overflow-auto p-4 bg-[hsla(0,0%,0%,.1)] rounded-md lg:max-w-4xl mx-2">
            <h2 className="text-xl font-bold">Open Source</h2>
                <p>
                    hdrop is developed in the open on <a className="underline underline-offset-2" href="https://github.com/ZitaneLabs/hdrop" target="_blank">github.com/ZitaneLabs/hdrop</a>. Feel free to contribute!
                </p>
                <br />
                <h2 className="text-xl font-bold">Crypto APIs</h2>
                <p>
                    hdrop exclusively uses the <code>WebCrypto</code> API provided by the browser to encrypt files on your own device, before sending them to the server. We have taken great care to ensure that the encryption process is as secure as possible, and that the server never has access to your files or their contents.
                </p>
                <br />
                <h2 className="text-xl font-bold">File Storage</h2>
                <p>
                    Files are stored on the server for a maximum of 24 hours, after which they are automatically deleted. File metadata such as file name, creation date, etc. as well as the full file contents are end-to-end encrypted.
                </p>
                <br />
                <h2 className="text-xl font-bold">File Access</h2>
                <p>
                    Files are only accessible by anyone who has both the link to the file and the password. The password is generated on the client using a secure random source provided by the <code>WebCrypto</code> API. The secure password is then used to derive the cryptographic key using the <code>PBKDF2</code> key derivation algorithm. The password itself is not stored on the server, and we do not even hand out the encrypted file contents until the user proves knowledge of the password by solving a cryptographic challenge.
                </p>
                <br />
                <h2 className="text-xl font-bold">Further Information</h2>
                <p>
                    The full details of our security model can be found in the <a className="underline underline-offset-2" href="https://github.com/ZitaneLabs/hdrop" target="_blank">GitHub repository</a>.
                </p>
            </p>
        </main>
    )
}
