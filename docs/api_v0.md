# API Spec

This is the initial API spec for hdrop, which was used for the MVP. It has been replaced by the [API v1](./api_v1.md) spec.

### API v0

##### Error Response Format
```json
{
    "status": "error",
    "data": {
        "reason": "Error reason.",
    }
}
```

#### `POST /v0/files`
> Upload a file.

##### Request
> Type: `multipart/form-data`

| Field            | Type     | Description                               |
| ---------------- | -------- | ----------------------------------------- |
| `file_data`      | `binary` | Raw encrypted file bytes                  |
| `file_name_data` | `string` | Base64 encoded encrypted file name        |
| `file_name_hash` | `string` | Hex encoded SHA-256 hash of the file name |
| `iv`             | `string` | Base64 encoded IV                         |
| `salt`           | `string` | Base64 encoded salt                       |

##### Response
> Type: `application/json`

```json
{
    "status": "success",
    "data": {
        "access_token": "abc123",
        "update_token": "qec61jytfzj",
    }
}
```

#### `POST /v0/files/{access_token}/expiry`
> Update the expiration time for a file.

##### Request
> Type: `application/json`

```json
{
    "expiry": 3600
}
```

###### URL Parameters

| Name           | Type     | Description                |
| -------------- | -------- | -------------------------- |
| `update_token` | `string` | Needed for authentication. |

##### Response (Success)
> Type: `application/json`

```json
{
    "status": "success"
}
```

#### `DELETE /v0/files/{access_token}`
> Delete a file.

##### Request

###### URL Parameters

| Name           | Type     | Description                |
| -------------- | -------- | -------------------------- |
| `update_token` | `string` | Needed for authentication. |

##### Response
> Type: `application/json`

```json
{
    "status": "success"
}
```

#### `GET /v0/files/{access_token}`
> Retrieve file metadata.

##### Response
> Type: `application/json`

```json
{
    "status": "success",
    "data": {
        "file_url": "https://example.com/file",
        "file_name_data": "bHVsdWx1LnR4dA==",
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```

#### `GET /v0/files/{access_token}/raw`
> Retrieve the raw encrypted file bytes.

##### Response
> Success Type: `application/octet-stream`
> Error Type: `application/json`

### `GET /v0/files/{access_token}/challenge`
> Get a challenge for a file.

##### Response
> Type: `application/json`

```json
{
    "status": "success",
    "data": {
        "challenge": "bHVsdWx1LnR4dA==",
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```

#### `POST /v0/files/{access_token}/challenge`
> Verify a challenge for a file.

##### Request
> Type: `application/json`

```json
{
    "challenge": "f1fc1ddfba46cc672a56f09a9f3467d75a53b96fc19d78ac0b5fd8e53d272bcc",
}
```

##### Response
> Type: `application/json`

```json
{
    "status": "success",
    "data": {
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```
