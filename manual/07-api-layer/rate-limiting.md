# Rate Limiting

> Sliding window rate limiting for API endpoints

**Status:** 🔄 Planned

---

## Purpose

Prevent API abuse and ensure fair resource usage.

---

## Rate Limits

### Per-User Tiers

| Tier | Requests/Minute | Requests/Hour |
|------|-----------------|---------------|
| Anonymous | 10 | 100 |
| Standard | 100 | 1,000 |
| Admin | 1,000 | 10,000 |

**Tier determination:** Based on authenticated user's role.

---

## Algorithm

### Sliding Window

```
Track requests in time buckets:
- Window: 1 minute (60 seconds)
- Buckets: 6 × 10-second buckets
- Slide: Remove old buckets, add new

Advantages:
- Smooth rate limiting
- No burst spikes at window boundaries
- Memory efficient
```

### Implementation

```rust
pub struct RateLimiter {
    requests: HashMap<UserId, VecDeque<Timestamp>>,
    limit: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn check(&mut self, user_id: &UserId) -> bool {
        let now = current_timestamp();
        let cutoff = now - self.window;
        
        // Get user's request history
        let history = self.requests
            .entry(user_id.clone())
            .or_insert_with(VecDeque::new);
        
        // Remove old requests
        while let Some(&ts) = history.front() {
            if ts < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
        
        // Check limit
        if history.len() >= self.limit {
            return false;  // Rate limit exceeded
        }
        
        // Add current request
        history.push_back(now);
        true
    }
}
```

---

## Response Headers

### On Success

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1704067260
```

**Fields:**
- `Limit` - Maximum requests in window
- `Remaining` - Requests remaining
- `Reset` - Unix timestamp when window resets

### On Limit Exceeded

**Status:** `429 Too Many Requests`

```json
{
  "error": {
    "type": "RateLimitExceeded",
    "message": "Rate limit exceeded. Try again in 45 seconds.",
    "retry_after": 45,
    "limit": 100,
    "window": 60
  }
}
```

**Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704067260
Retry-After: 45
```

---

## Exemptions

### Bypass Rate Limiting

**Admin users** may have higher limits or exemptions.

**Whitelist:** Specific IPs or API keys can be exempted.

### Per-Endpoint Limits

**Different limits for expensive operations:**
- GET requests: Standard limits
- POST/PUT/DELETE: Lower limits (e.g., 50/minute)

---

## Client Best Practices

### Respect Headers

```python
response = requests.get(url, headers={"Authorization": f"Bearer {token}"})

# Check rate limit
remaining = int(response.headers.get("X-RateLimit-Remaining", 0))
if remaining < 10:
    print("Warning: Approaching rate limit")

# Handle 429
if response.status_code == 429:
    retry_after = int(response.headers.get("Retry-After", 60))
    time.sleep(retry_after)
    response = requests.get(url)  # Retry
```

### Implement Backoff

```javascript
async function apiCall(url, retries = 3) {
  for (let i = 0; i < retries; i++) {
    const response = await fetch(url);
    
    if (response.status === 429) {
      const retryAfter = response.headers.get('Retry-After');
      await sleep(retryAfter * 1000);
      continue;
    }
    
    return response;
  }
  throw new Error('Rate limit exceeded after retries');
}
```

---

## Monitoring

### Metrics

**Track:**
- Requests per user per minute
- 429 response rate
- Average requests per user
- Peak usage times

**Alerts:**
- User consistently hitting limits
- Unusual spike in requests
- Potential abuse patterns

---

**See also:**
- [Authentication](authentication.md) - User identification
- [RESTful Endpoints](restful-endpoints.md) - API reference
