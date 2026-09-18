# Security
> This document describes the hdrop security model in detail.

## Crypto APIs

hdrop exclusively uses `WebCrypto` APIs provided by the browser for all cryptographic operations.

## File Storage

### Key Derivation (Client)

1. Generate a random salt `S` with a size of 16 bytes
2. Derive an AES-256-GCM key `K` from the password `P` using PBKDF2-HMAC-SHA-256 (`600_000` iterations by default).

Derived AES keys are non-extractable and permit only `encrypt` and `decrypt`. The client does not export keys.

### Crypto Configuration

Crypto settings must be decimal integers. Unset settings use the defaults below. Upload and download clients must use the same iteration count, which is not stored in file metadata.

Explicit empty, malformed, fractional, or out-of-range values throw before use.

| Setting                         | Default/minimum | Maximum       |
| ------------------------------- | --------------- | ------------- |
| `NEXT_PUBLIC_PBKDF2_ITERATIONS` | 600,000         | 4,294,967,295 |
| `NEXT_PUBLIC_PASSWORD_BYTES`    | 32              | 65,536        |
| `NEXT_PUBLIC_CHALLENGE_BYTES`   | 32              | 65,536        |

Password and challenge lengths count random bytes, before Base64 encoding the password. Their upper bound is the per-call `crypto.getRandomValues()` limit.

### File Encryption (Client)

File contents, file names, and challenges are encrypted before upload.

All three object types use AES-256-GCM with 96-bit IVs and 128-bit authentication tags. Fixed Additional Authenticated Data (encoded as UTF-8 with `TextEncoder`) provides domain separation:

| Object        | AAD                  |
| ------------- | -------------------- |
| File contents | `hdrop/v1/file`      |
| File name     | `hdrop/v1/filename`  |
| Challenge     | `hdrop/v1/challenge` |

AAD is public and authenticated alongside the ciphertext. The client supplies the exact same domain during encryption and decryption. It never takes the domain from server metadata.

Decrypting under a different domain fails authentication. This means if a malicious server substitutes both ciphertext and its matching IV, the authentication fails.

1. Generate a fresh 96-bit (12-byte) base initialization vector `IV` using `crypto.getRandomValues()`
   - The frontend provides **two** static XOR masks to generate different IVs for the file name and the challenge (based on the random IV).[^1]
   - Producing IV, Name_IV, Challenge_IV
2. Encrypt file data `Fd` using `AES-256-GCM-ENC(IV, K, Fd, "hdrop/v1/file")`
   - Producing encrypted file data `EFd`
3. Encrypt file name `Fn` using `AES-256-GCM-ENC(Name_IV, K, Fn, "hdrop/v1/filename")`
   - Producing encrypted file name `EFn`
4. Generate and encrypt file challenge `Fc` (32 random bytes by default) using `AES-256-GCM-ENC(Challenge_IV, K, Fc, "hdrop/v1/challenge")`
   - Producing encrypted file challenge `EFc`
5. Hash file challenge `Fc` using `SHA-256(Fc)`
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

1. Retrieve challenge data (`EFc`, `S`, `IV`) from server
2. Derive the same AES-256-GCM key `K` using PBKDF2-HMAC-SHA-256 and the configured iteration count

### Challenge

#### Client

1. Decrypt the encrypted file challenge `EFc` using `AES-256-GCM-DEC(Challenge_IV, K, EFc, "hdrop/v1/challenge")`
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

1. Decrypt encrypted file data `EFd` using `AES-256-GCM-DEC(IV, K, EFd, "hdrop/v1/file")`
   - Producing file data `Fd`
2. Decrypt encrypted file name `EFn` using `AES-256-GCM-DEC(Name_IV, K, EFn, "hdrop/v1/filename")`
   - Producing file name `Fn`

#### Notes

The client can - given the right access token - always request the encrypted file challenge. The challenge is mainly a solution to improve UX by avoiding having to download the entirety of encrypted file contents before attempting decryption. It also has the added benefit of completely denying access to the encrypted file data and name for people without the right password.

[^1]: The two fixed 12-byte XOR masks are non-zero and different. Thus `IV`, `IV XOR filename_mask`, and `IV XOR challenge_mask` are deterministically pairwise distinct **within one encryption context**. The masks are public domain-separation constants/tags. They are **not** used as secret pads. They allow the client to reconstruct all three IVs from one stored base IV. Runtime checks reject invalid IV/mask lengths and zero masks. Regression tests also verify the fixed masks differ. This does not provide global IV uniqueness: independent contexts can have overlapping IV sets. AES-GCM **must never** encrypt distinct messages with the same key and IV, and AAD does not remove that requirement. Each upload generates a fresh random password and salt to derive a fresh key. Reusing an AES key across independent uploads would require separate nonce-collision prevention, the XOR construction alone is insufficient.
