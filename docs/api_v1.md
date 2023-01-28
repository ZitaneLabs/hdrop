# API Spec

### API v1

##### Error Response Format
```json
{
    "status": "error",
    "data": {
        "reason": "Error reason.",
    }
}
```

#### `POST /v1/files`
> Upload a file.

##### Request
```json
{
  "file_data": "SGVsbG8sIHdvcmxkIQ==",
  "file_name_data": "bHVsdWx1LnR4dA==",
  "file_extension": "txt",
  "iv": "MDAwMDAwMDAwMDAw",
  "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
}
```

##### Response
```json
{
    "status": "success",
    "data": {
        "access_token": "abc123",
        "update_token": "qec61jytfzj",
    }
}
```

#### `POST /v1/files/{access_token}/expiry`
> Update the expiration time for a file.

##### Request
```json
{
    "expiry_time": 3600
}
```

###### URL Parameters

| Name           | Type     | Description                |
| -------------- | -------- | -------------------------- |
| `update_token` | `string` | Needed for authentication. |

##### Response (Success)
```json
{
    "status": "success"
}
```

#### `DELETE /v1/files/{access_token}`
> Delete a file.

##### Request

###### URL Parameters

| Name           | Type     | Description                |
| -------------- | -------- | -------------------------- |
| `update_token` | `string` | Needed for authentication. |

##### Response

```json
{
    "status": "success"
}
```

#### `GET /v1/files/{access_token}`
> Retrieve a file.

##### Response (File Data)
```json
{
    "status": "success",
    "data": {
        "file_data": "SGVsbG8sIHdvcmxkIQ==",
        "file_url": null,
        "file_name_data": "bHVsdWx1LnR4dA==",
    }
}
```

##### Response (File URL)
```json
{
    "status": "success",
    "data": {
        "file_data": null,
        "file_url": "https://example.com/file",
        "file_name_data": "bHVsdWx1LnR4dA==",
    }
}
```

### `GET /v1/files/{access_token}/challenge`
> Get a challenge for a file.

##### Response
```json
{
    "status": "success",
    "data": {
        "challenge": "bHVsdWx1LnR4dA==",
    }
}
```

#### `POST /v1/files/{access_token}/challenge`
> Verify a challenge for a file.

##### Request
```json
{
    "challenge": "f1fc1ddfba46cc672a56f09a9f3467d75a53b96fc19d78ac0b5fd8e53d272bcc",
}
```

##### Response
```json
{
    "status": "success",
    "data": {
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```