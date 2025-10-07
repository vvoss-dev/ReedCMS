# API Layer (Layer 07)

> RESTful HTTP API with security matrix and rate limiting

**Status:** 🔄 In Progress  
**Implementation:** REED-07-01 to REED-07-02

---

## Overview

The API Layer provides RESTful HTTP endpoints for external integrations, featuring role-based access control, rate limiting, and JSON responses.

**Note:** This layer is currently in development. Documentation reflects planned features.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│         External Application                     │
│   curl -H "Authorization: Bearer token..."      │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         API Gateway                              │
│   /api/v1/*                                      │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Authentication                           │
│   Bearer Token → Validate Session                │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Authorisation (Security Matrix)          │
│   Check Permission: text[r--], user[rwx]         │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         Rate Limiting                            │
│   Check: < 100 requests/minute                   │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         API Handler                              │
│   Process Request → ReedBase                     │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│         JSON Response                            │
│   { data, source, cached, timestamp }            │
└──────────────────────────────────────────────────┘
```

---

## Core Concepts

### RESTful Design

**Resources:**
- Text content (`/api/v1/text`)
- Routes (`/api/v1/routes`)
- Metadata (`/api/v1/meta`)
- Users (`/api/v1/users`)
- Roles (`/api/v1/roles`)

**Methods:**
- `GET` - Retrieve resource
- `POST` - Create resource
- `PUT` - Update resource
- `DELETE` - Delete resource

### Bearer Token Authentication

**Header format:**
```
Authorization: Bearer <session_token>
```

**Obtained from login:**
```bash
curl -X POST https://example.com/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "secret"}'

# Response:
{
  "token": "abc123...",
  "expires_at": 1704067200
}
```

### Security Matrix

**Permission mapping:**
```
GET    /api/v1/text     → text[r--]
POST   /api/v1/text     → text[-w-]
PUT    /api/v1/text     → text[-w-]
DELETE /api/v1/text     → text[--x]

GET    /api/v1/users    → user[r--]
POST   /api/v1/users    → user[-w-]
DELETE /api/v1/users    → user[--x]
```

**User role check:**
```
User: editor
Roles: text[rwx], content[rwx]
Request: GET /api/v1/text
Permission required: text[r--]
Result: ✅ Allowed
```

### Rate Limiting

**Per-user limits:**
- Standard: 100 requests/minute
- Admin: 1000 requests/minute
- Anonymous: 10 requests/minute

**Algorithm:** Sliding window

**Response headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1704067200
```

---

## API Endpoints (Planned)

### Text Operations

#### GET /api/v1/text/{key}

**Retrieve text content**

**Parameters:**
- `key` (path) - Text key (e.g., `page.title`)
- `lang` (query) - Language code (default: `en`)

**Example:**
```bash
curl -H "Authorization: Bearer token..." \
     "https://example.com/api/v1/text/page.title?lang=de"
```

**Response:**
```json
{
  "data": "Willkommen",
  "source": "reedbase::get::text",
  "cached": true,
  "timestamp": 1704067200
}
```

#### POST /api/v1/text

**Create text entry**

**Request:**
```json
{
  "key": "page.title",
  "value": "Welcome",
  "language": "en",
  "description": "Homepage title"
}
```

**Response:**
```json
{
  "data": "Welcome",
  "source": "reedbase::set::text",
  "cached": false,
  "timestamp": 1704067200
}
```

#### PUT /api/v1/text/{key}

**Update text entry**

**Request:**
```json
{
  "value": "Welcome to ReedCMS",
  "language": "en"
}
```

#### DELETE /api/v1/text/{key}

**Delete text entry**

**Parameters:**
- `key` (path) - Text key
- `lang` (query) - Language code

---

### User Operations

#### GET /api/v1/users

**List all users**

**Query parameters:**
- `limit` - Results per page (default: 50)
- `offset` - Pagination offset
- `active` - Filter by active status

**Response:**
```json
{
  "data": [
    {
      "username": "admin",
      "email": "admin@example.com",
      "roles": ["admin", "editor"],
      "is_active": true,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

#### GET /api/v1/users/{username}

**Get user details**

#### POST /api/v1/users

**Create user**

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

#### DELETE /api/v1/users/{username}

**Delete user**

---

### Route Operations

#### GET /api/v1/routes

**List all routes**

#### GET /api/v1/routes/{layout}

**Get route for layout**

**Parameters:**
- `layout` (path) - Layout name
- `lang` (query) - Language code

**Example:**
```bash
curl "https://example.com/api/v1/routes/knowledge?lang=de"
```

**Response:**
```json
{
  "data": "wissen",
  "layout": "knowledge",
  "language": "de",
  "description": "German knowledge page"
}
```

---

## Response Format

### Success Response

```json
{
  "data": <result>,
  "source": "service::function",
  "cached": true|false,
  "timestamp": 1704067200,
  "metrics": {
    "execution_time_ms": 15,
    "cache_hits": 1
  }
}
```

### Error Response

```json
{
  "error": {
    "type": "NotFound",
    "message": "Key not found: page.title@en",
    "code": "ERR_NOT_FOUND"
  },
  "timestamp": 1704067200
}
```

### Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `204 No Content` - Success, no body
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Permission denied
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

---

## Authentication Flow

### Login

```bash
POST /api/v1/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}
```

**Response:**
```json
{
  "token": "abc123def456...",
  "expires_at": 1704070800,
  "user": {
    "username": "admin",
    "roles": ["admin"]
  }
}
```

### Using Token

```bash
GET /api/v1/text/page.title
Authorization: Bearer abc123def456...
```

### Logout

```bash
POST /api/v1/logout
Authorization: Bearer abc123def456...
```

---

## Rate Limiting

### Per-User Limits

**Standard user:**
```
100 requests per minute
1000 requests per hour
```

**Admin user:**
```
1000 requests per minute
10000 requests per hour
```

**Anonymous (no auth):**
```
10 requests per minute
100 requests per hour
```

### Headers

**Response includes:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1704067260
```

### 429 Response

```json
{
  "error": {
    "type": "RateLimitExceeded",
    "message": "Too many requests. Try again in 45 seconds.",
    "retry_after": 45
  }
}
```

---

## Security Matrix

### Endpoint Permissions

| Endpoint | Method | Permission Required |
|----------|--------|---------------------|
| `/api/v1/text` | GET | `text[r--]` |
| `/api/v1/text` | POST | `text[-w-]` |
| `/api/v1/text` | PUT | `text[-w-]` |
| `/api/v1/text` | DELETE | `text[--x]` |
| `/api/v1/users` | GET | `user[r--]` |
| `/api/v1/users` | POST | `user[-w-]` |
| `/api/v1/users` | DELETE | `user[--x]` |
| `/api/v1/roles` | GET | `role[r--]` |
| `/api/v1/roles` | POST | `role[-w-]` |

### Permission Checks

**Process:**
1. Extract Bearer token from header
2. Validate session (token valid + not expired)
3. Get user from session
4. Map endpoint to resource permission
5. Check if user has required permission
6. Allow or deny request

---

## Integration Examples

### Python Client

```python
import requests

# Login
response = requests.post(
    "https://example.com/api/v1/login",
    json={"username": "admin", "password": "secret"}
)
token = response.json()["token"]

# Get text
response = requests.get(
    "https://example.com/api/v1/text/page.title?lang=en",
    headers={"Authorization": f"Bearer {token}"}
)
print(response.json()["data"])  # "Welcome"

# Create text
response = requests.post(
    "https://example.com/api/v1/text",
    headers={"Authorization": f"Bearer {token}"},
    json={
        "key": "page.subtitle",
        "value": "High-performance CMS",
        "language": "en"
    }
)
```

### JavaScript Client

```javascript
// Login
const login = await fetch('https://example.com/api/v1/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ username: 'admin', password: 'secret' })
});
const { token } = await login.json();

// Get text
const response = await fetch(
  'https://example.com/api/v1/text/page.title?lang=en',
  { headers: { 'Authorization': `Bearer ${token}` } }
);
const data = await response.json();
console.log(data.data);  // "Welcome"
```

---

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Authentication | < 100ms | Argon2 verification |
| Authorisation check | < 1ms | Permission lookup |
| Rate limit check | < 1ms | Sliding window |
| Data retrieval | < 100μs | ReedBase cache |
| JSON serialisation | < 1ms | serde_json |
| **Total** | < 110ms | First request (auth) |
| **Cached** | < 5ms | Subsequent requests |

---

## Documentation

- [RESTful Endpoints](restful-endpoints.md) - Complete API reference
- [Authentication](authentication.md) - Token-based auth
- [Security Matrix](security-matrix.md) - Permission system
- [Rate Limiting](rate-limiting.md) - Sliding window algorithm

---

## Related Layers

- **Layer 03 - Security:** User authentication and RBAC
- **Layer 06 - Server:** HTTP server infrastructure
- **Layer 02 - Data:** ReedBase for data operations

---

## Summary

The API Layer provides (planned):
- ✅ RESTful HTTP API design
- ✅ Bearer token authentication
- ✅ Security matrix (RBAC integration)
- ✅ Rate limiting (sliding window)
- ✅ JSON responses with metadata
- ✅ Complete CRUD operations
- ✅ Error handling with status codes
- ✅ < 5ms response time (cached)

**Status:** In development - Core features planned and specified.
