# Hausdrop
> Simple, self-hosted encrypted file transfer.

## API Spec

### API v1
Base URL: `/v1`

#### `PUT /file`
Upload a file.

##### Request
```json
{
  "file_data": "bHVsdWx1LnR4dA==",
  "file_name_data": "SGVsbG8sIHdvcmxkIQ==",
  "iv": "MDAwMDAwMDAwMDAw",
  "salt": "MDAwMDAwMDAwMDAwMDAwMA==",
}
```

##### Response (OK)
```json
{
    "status": "ok",
    "data": {
        "shortcode": "abc123"
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

#### `GET /file/{shortcode}/metadata`
Get metadata for a file.

##### Response (OK)
```json
{
    "status": "ok",
    "data": {
        "file_name_data": "SGVsbG8sIHdvcmxkIQ==",
        "file_extension": "txt",
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

#### `GET /file/{shortcode}`
Retrieve a file.

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