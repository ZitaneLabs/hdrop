# Security
> This document describes the hdrop security model in detail.

## Crypto APIs

hdrop exclusively uses `WebCrypto` APIs provided by the browser for all cryptographic operations.

## File Storage

### Key Derivation (Client)

1. Generate a random salt `S` with a size of 16 bytes
2. Derive a key `K` from the password `P` using `600_000` rounds of `PBKDF2(P, S)`

### File Encryption (Client)

1. Generate a random initialization vector `IV` with a size of 12 bytes
   - The frontend provides **two** static XOR masks to generate different IVs for the file name and the challenge (based on the random IV).[^1]
   - Producing IV, Name_IV, Challenge_IV
3. Encrypt file data `Fd` using `AES-256-GCM-ENC(IV, K, Fd)`
   - Producing encrypted file data `EFd`
4. Encrypt file name `Fn` using `AES-256-GCM-ENC(Name_IV, K, Fn)`
   - Producing encrypted file name `EFn`
5. Generate and encrypt 32 bytes long file challenge `Fc` using `AES-256-GCM-ENC(Challenge_IV, K, Fc)`
   - Producing encrypted file challenge `EFc`
7. Hash file challenge `Fc` using `SHA-256(Fc)`
   - Producing hashed file challenge `H(Fc)`

### File Upload

#### Client

1. Upload data to the server:
   - Encrypted file contents
   - Base64-encoded encrypted file name
   - Base64-encoded salt
   - Base64-encoded IV
   - Base64-encoded encrypted challenge data
   - Hashed challenge data
  
#### Server

1. Get a pair of tokens back:
   - Access Token `Ta` (guaranteed unique)
   - Update Token `Tu` (not unique, but sufficiently random)

#### Tokens

| Token             | Private | Usage                                           |
| ----------------- | ------- | ----------------------------------------------- |
| Access Token `Ta` | `no`    | `Used for file retrival by third parties`       |
| Update Token `Tu` | `yes`   | `Used for authenticating the original uploader` |

### File Manipulation
> Update expiry and manual deletion can only be done by the owner (original uploader).

### Update expiry time
> Expiry time is recalculated based on original storage date.<br>
> The Update Token `Tu` is used to authenticate the caller.

### Delete file
> The Update Token `Tu` is used to authenticate the caller.

## File Retrieval

### Key Derivation

1. Retrieve challenge data (`EFn`, `S`, `IV`) from server
2. Derive a key `K` from the password `P` using `600_000` rounds of `PBKDF2(P, S)`

### Challenge

#### Client

1. Decrypt the encrypted file challenge `EFc` using `AES-256-GCM-DEC(Challenge_IV, K, EFc)`
   - Producing file challenge `Fc'` (== `Fc`, if successful)
2. Hash file challenge `Fc'` using `SHA-256(Fc')`
   - Producing challenge solution `H(Fc')` (== `H(Fc)`, if successful)
3. Send challenge solution `H(Fc')` to server

#### Server

1. Check challenge solution `H(Fc')` against hashed file challenge `H(Fc)`
   - `IF H(Fc') == H(Fc)`: Challenge solved, respond with success and encrypted file name `EFn`
   - `IF H(Fc') ≠ H(Fc)`: Challenge failed, respond with error and deny download
> The challenge solution `H(Fc')` acts as an authorization bearer token for the download

### File Decryption

1. Decrypt encrypted file data `EFd` using `AES-256-GCM-DEC(IV, K, EFd)`
   - Producing file data `Fd`
2. Decrypt encrypted file name `EFn` using `AES-256-GCM-DEC(Name_IV, K, EFn)`
   - Producing file name `Fn`

#### Notes

The client can - given the right access token - always request the encrypted file challenge. The challenge is mainly a solution to improve UX by avoiding having to download the entirety of encrypted file contents before attempting decryption. It also has the added benefit of completely denying access to the encrypted file data and name for people without the right password.

[^1]: The GCM mode of operation should **never** reuse the same IV to encrypt other related data/messages. Therefore two XOR masks are provided instead of generating three different IVs. This ensures 100% that the IVs for data, name and challenge are all different. The XOR masks are located in the frontend to avoid storing two additional rows for the IVs in the backend database.
