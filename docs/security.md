# Security
> This document describes the hdrop security model in detail.

## Crypto APIs

hdrop exclusively uses `WebCrypto` APIs for all cryptographic operations.

## File Storage

### Key Derivation (Client)

1. Generate a random salt `S` with a size of 16 bytes
2. Derive a key `K` from the password `P` using `100_000` rounds of `PBKDF2(P, S)`

### File Encryption (Client)

1. Generate a random initialization vector `IV` with a size of 12 bytes
2. Encrypt file contents `Fc` using `AES-256-GCM-ENC(IV, K, Fc)`
   - Producing encrypted file contents `EFc`
3. Encrypt file name `Fn` using `AES-256-GCM-ENC(IV, K, Fn)`
   - Producing encrypted file name `EFn`
4. Hash file name `Fn` using `SHA-256(Fn)`
   - Producing hashed file name `C`

### File Upload

#### Client

1. Upload data to the server:
   - Encrypted file contents
   - Base64-encoded encrypted file name
   - Base64-encoded salt
   - Base64-encoded IV
   - Hashed file name
  
#### Server

1. Get a pair of tokens back:
   - Access Token `Ta` (guaranteed unique)
   - Update Token `Tu` (not unique, but sufficiently random)

#### Tokens

| Token             | Private | Usage                                           |
| ----------------- | ------- | ----------------------------------------------- |
| Access Token `Ta` | `no`    | `Used for file retrival by third parties`       |
| Update Token `Tu` | `yes`   | `Used for authenticating the original uploader` |

## File Manipulation
> A file can only be manipulated by the owner (original uploader).

### Update expiry time
> Expiry time is recalculated based on original storage date.<br>
> The Update Token `Tu` is used to authenticate the caller.

### Delete file
> The Update Token `Tu` is used to authenticate the caller.

## File Retrieval

### Key Derivation

1. Retrieve challenge data (`EFn`, `S`, `IV`) from server
2. Derive a key `K` from the password `P` using `100_000` rounds of `PBKDF2(P, S)`

### Challenge

#### Client

1. Decrypt the encrypted file name `EFn` using `AES-256-GCM-DEC(IV, K, EFn)`
   - Producing challenge data `Cd`
2. Hash challenge data `Cd` using `SHA-256(Cd)`
   - Producing challenge solution `Cs`
3. Send challenge solution `Cs` to server

#### Server

1. Check challenge solution `Cs` against hashed file name `C`
   - `IF Cs == C`: Challenge solved, respond with success
   - `IF Cs != C`: Challenge failed, respond with error

### File Decryption

1. Decrypt encrypted file contents `EFc` using `AES-256-GCM-DEC(IV, K, EFc)`
   - Producing file contents `Fc`
2. Decrypt encrypted file name `EFn` using `AES-256-GCM-DEC(IV, K, EFn)`
   - Producing file name `Fn`

#### Notes

The client can - given the right access token - always request encrypted file data, without having to solve the challenge first. The challenge does not add to or compromise the existing security measures, it is merely a simple solution to avoid having to download the entirety of encrypted file contents before attempting decryption.