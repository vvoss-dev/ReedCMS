# API Authentication

> Bearer token authentication for API access

**Status:** 🔄 Planned

---

## Authentication Flow

```
1. Login → POST /api/v1/login
2. Receive token
3. Use token in Authorization header
4. Token valid for 1 hour
5. Refresh or re-login
```

---

## Login

### Request

```bash
POST /api/v1/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secret"
}
```

### Response

```json
{
  "token": "abc123def456...",
  "expires_at": 1704070800,
  "user": {
    "username": "admin",
    "roles": ["admin", "editor"]
  }
}
```

**Token format:** Random 32-character alphanumeric string  
**Expiration:** 1 hour (3600 seconds)

---

## Using Token

### Header Format

```
Authorization: Bearer <token>
```

### Example

```bash
curl -H "Authorization: Bearer abc123def456..." \
     https://example.com/api/v1/text/page.title?lang=en
```

---

## Token Validation

**Server checks:**
1. Token exists in session store
2. Token not expired
3. User associated with token
4. User is active

**On failure:** `401 Unauthorized`

---

## Logout

```bash
POST /api/v1/logout
Authorization: Bearer abc123def456...
```

**Effect:** Invalidates token immediately

---

## Token Refresh

**Not implemented yet.** Current approach:
- Token expires after 1 hour
- Re-login required

**Future:** Refresh token endpoint

---

## Security

**Token storage (client):**
- ✅ Secure: Environment variable, secure cookie
- ❌ Insecure: localStorage, sessionStorage (XSS risk)

**Transmission:**
- ✅ Always use HTTPS in production
- ❌ Never use HTTP (token visible)

---

**See also:**
- [Security Layer - Authentication](../03-security-layer/authentication.md) - Session management
- [RESTful Endpoints](restful-endpoints.md) - API reference
