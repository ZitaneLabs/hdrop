# Hausdrop
> Simple, self-hosted encrypted file transfer.

## API Spec

### API v1

#### `PUT /v1/file`
> Upload a file.

##### Request
```json
{
  "file_data": "bHVsdWx1LnR4dA==",
  "file_name_data": "SGVsbG8sIHdvcmxkIQ==",
  "file_extension": "txt",
  "iv": "MDAwMDAwMDAwMDAw",
  "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
}
```

##### Response (OK)
```json
{
    "status": "ok",
    "data": {
        "access_token": "abc123",
        "update_token": "qec61jytfzj",
    }
}
```

##### Response (Error)
```json
{
    "status": "error",
    "data": {
        "reason": "File too big (max 5GB)"
    }
}
```

#### `POST /v1/file/{access_token}/expiry`
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

##### Response (OK)
```json
{
    "status": "ok"
}
```

##### Response (Error)
```json
{
    "status": "error",
    "data": {
        "reason": "Invalid update token"
    }
}
```

#### `DELETE /v1/file/{access_token}`
> Delete a file.

##### Request

###### URL Parameters

| Name           | Type     | Description                |
| -------------- | -------- | -------------------------- |
| `update_token` | `string` | Needed for authentication. |

##### Response (OK)

```json
{
    "status": "ok"
}
```

##### Response (Error)

```json
{
    "status": "error",
    "data": {
        "reason": "Invalid update token"
    }
}
```

#### `GET /v1/file/{access_token}`
> Retrieve a file.

##### Response (OK)
```json
{
    "status": "ok",
    "data": {
        "file_data": "bHVsdWx1LnR4dA==",
        "file_name_data": "SGVsbG8sIHdvcmxkIQ==",
        "file_extension": "txt",
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```

##### Response (Error)
```json
{
    "status": "error",
    "data": {
        "reason": "File not found"
    }
}
```

#### `GET /v1/file/{access_token}/metadata`
> Get metadata for a file.<br>

##### Response (OK)
```json
{
    "status": "ok",
    "data": {
        "file_name_data": "SGVsbG8sIHdvcmxkIQ==",
        "file_extension": "txt",
        "iv": "MDAwMDAwMDAwMDAw",
        "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
    }
}
```

##### Response (Error)
```json
{
    "status": "error",
    "data": {
        "reason": "File not found"
    }
}
```