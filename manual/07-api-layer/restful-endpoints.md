# RESTful Endpoints

> Complete API endpoint reference

**Status:** 🔄 Planned - Endpoints specified but not yet implemented.

---

## Base URL

```
https://example.com/api/v1
```

---

## Authentication

**All endpoints require authentication:**
```
Authorization: Bearer <token>
```

**See:** [Authentication](authentication.md)

---

## Text Endpoints

### GET /api/v1/text/{key}

Retrieve text content.

**Parameters:**
- `key` (path) - Text key
- `lang` (query, optional) - Language code (default: `en`)

**Response:** `200 OK`
```json
{
  "data": "Welcome",
  "source": "reedbase::get::text",
  "cached": true,
  "timestamp": 1704067200
}
```

### POST /api/v1/text

Create text entry.

**Request:**
```json
{
  "key": "page.title",
  "value": "Welcome",
  "language": "en",
  "description": "Homepage title"
}
```

**Response:** `201 Created`

### PUT /api/v1/text/{key}

Update text entry.

**Request:**
```json
{
  "value": "New value",
  "language": "en"
}
```

**Response:** `200 OK`

### DELETE /api/v1/text/{key}

Delete text entry.

**Parameters:**
- `key` (path) - Text key
- `lang` (query) - Language code

**Response:** `204 No Content`

---

## User Endpoints

### GET /api/v1/users

List all users.

**Query:**
- `limit` - Results per page (default: 50, max: 100)
- `offset` - Pagination offset
- `active` - Filter: `true`, `false`, or omit

**Response:** `200 OK`
```json
{
  "data": [
    {
      "username": "admin",
      "email": "admin@example.com",
      "roles": ["admin"],
      "is_active": true
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### GET /api/v1/users/{username}

Get user details.

### POST /api/v1/users

Create user.

**Request:**
```json
{
  "username": "jdoe",
  "email": "john@example.com",
  "password": "secure123",
  "firstname": "John",
  "lastname": "Doe"
}
```

**Response:** `201 Created`

### PUT /api/v1/users/{username}

Update user.

### DELETE /api/v1/users/{username}

Delete user.

**Response:** `204 No Content`

---

## Route Endpoints

### GET /api/v1/routes

List all routes.

### GET /api/v1/routes/{layout}

Get route for layout.

**Parameters:**
- `layout` (path) - Layout name
- `lang` (query) - Language code

**Response:** `200 OK`
```json
{
  "data": "wissen",
  "layout": "knowledge",
  "language": "de"
}
```

### POST /api/v1/routes

Create route.

---

## Role Endpoints

### GET /api/v1/roles

List all roles.

### GET /api/v1/roles/{rolename}

Get role details.

### POST /api/v1/roles

Create role.

### PUT /api/v1/roles/{rolename}

Update role.

### DELETE /api/v1/roles/{rolename}

Delete role.

---

## Error Responses

### 400 Bad Request
```json
{
  "error": {
    "type": "ValidationError",
    "message": "Invalid email format",
    "field": "email"
  }
}
```

### 401 Unauthorized
```json
{
  "error": {
    "type": "Unauthorized",
    "message": "Missing or invalid token"
  }
}
```

### 403 Forbidden
```json
{
  "error": {
    "type": "PermissionDenied",
    "message": "Insufficient permissions: text[r--] required",
    "required": "text[r--]"
  }
}
```

### 404 Not Found
```json
{
  "error": {
    "type": "NotFound",
    "message": "Key not found: page.title@en"
  }
}
```

### 429 Too Many Requests
```json
{
  "error": {
    "type": "RateLimitExceeded",
    "message": "Rate limit exceeded",
    "retry_after": 45
  }
}
```

---

**See also:**
- [Authentication](authentication.md) - Token management
- [Security Matrix](security-matrix.md) - Permission requirements
- [Rate Limiting](rate-limiting.md) - Request limits
