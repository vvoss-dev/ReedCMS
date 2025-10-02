# REED-11-03: External API Bridges (Social Media Integration)

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files as `{name}_test.rs`

## Ticket Information
- **ID**: REED-11-03
- **Title**: External API Bridges (Social Media Integration)
- **Layer**: Extension Layer (REED-11)
- **Priority**: Low
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-11-01 (Hook System), REED-04-10 (Agent Commands)

## Objective
Implement bidirectional API bridges for social media platforms enabling automatic posting and content synchronisation.

## Supported Platforms

### Mastodon
- Post status updates
- Post threads
- Upload media
- Schedule posts
- Get account info

### Twitter/X
- Post tweets
- Post threads
- Upload media
- Rate limit handling

### LinkedIn
- Post articles
- Company page posting
- Personal profile posting

## Configuration

**.reed/integrations.csv**
```csv
integration_id|platform|instance|access_token_encrypted|config|status|created_at
mastodon-main|mastodon|social.example.com|[encrypted]|visibility=public|active|2025-10-02T...
twitter-main|twitter||[encrypted]|{}|active|2025-10-02T...
linkedin-company|linkedin||[encrypted]|company_id=12345|active|2025-10-02T...
```

## Implementation

### File Structure
```
src/reedcms/
├── extensions/
│   ├── integrations/
│   │   ├── mod.rs
│   │   ├── registry.rs         # Integration registry
│   │   ├── mastodon.rs         # Mastodon API
│   │   ├── twitter.rs          # Twitter API
│   │   ├── linkedin.rs         # LinkedIn API
│   │   └── traits.rs           # Common traits
```

### Integration Trait

```rust
// src/reedcms/extensions/integrations/traits.rs

pub trait SocialIntegration {
    /// Post text content.
    fn post_text(&self, content: &str, params: &PostParams) -> ReedResult<PostResult>;
    
    /// Post thread (multiple connected posts).
    fn post_thread(&self, posts: &[String], params: &PostParams) -> ReedResult<Vec<PostResult>>;
    
    /// Upload media.
    fn upload_media(&self, media: &MediaUpload) -> ReedResult<String>;
    
    /// Test connection.
    fn test_connection(&self) -> ReedResult<bool>;
}

#[derive(Debug)]
pub struct PostParams {
    pub visibility: Option<String>,
    pub reply_to: Option<String>,
    pub media_ids: Vec<String>,
}

#[derive(Debug)]
pub struct PostResult {
    pub post_id: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct MediaUpload {
    pub path: String,
    pub description: Option<String>,
}
```

### Mastodon Implementation

```rust
// src/reedcms/extensions/integrations/mastodon.rs

use super::traits::{SocialIntegration, PostParams, PostResult};

pub struct MastodonIntegration {
    instance: String,
    access_token: String,
}

impl MastodonIntegration {
    pub fn new(instance: String, access_token: String) -> Self {
        Self { instance, access_token }
    }
    
    fn api_url(&self, endpoint: &str) -> String {
        format!("https://{}/api/v1/{}", self.instance, endpoint)
    }
}

impl SocialIntegration for MastodonIntegration {
    fn post_text(&self, content: &str, params: &PostParams) -> ReedResult<PostResult> {
        let client = reqwest::blocking::Client::new();
        
        let mut body = serde_json::json!({
            "status": content,
        });
        
        if let Some(ref visibility) = params.visibility {
            body["visibility"] = serde_json::json!(visibility);
        }
        
        if let Some(ref reply_to) = params.reply_to {
            body["in_reply_to_id"] = serde_json::json!(reply_to);
        }
        
        if !params.media_ids.is_empty() {
            body["media_ids"] = serde_json::json!(params.media_ids);
        }
        
        let response = client
            .post(self.api_url("statuses"))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .map_err(|e| ReedError::ExternalServiceError {
                service: "mastodon".to_string(),
                reason: e.to_string(),
            })?;
            
        if !response.status().is_success() {
            return Err(ReedError::ExternalServiceError {
                service: "mastodon".to_string(),
                reason: format!("HTTP {}: {}", response.status(), response.text().unwrap_or_default()),
            });
        }
        
        let json: serde_json::Value = response.json().map_err(|e| ReedError::ExternalServiceError {
            service: "mastodon".to_string(),
            reason: format!("JSON parse error: {}", e),
        })?;
        
        Ok(PostResult {
            post_id: json["id"].as_str().unwrap_or("").to_string(),
            url: json["url"].as_str().unwrap_or("").to_string(),
            created_at: json["created_at"].as_str().unwrap_or("").to_string(),
        })
    }
    
    fn post_thread(&self, posts: &[String], params: &PostParams) -> ReedResult<Vec<PostResult>> {
        let mut results = Vec::new();
        let mut reply_to = params.reply_to.clone();
        
        for post_content in posts {
            let post_params = PostParams {
                visibility: params.visibility.clone(),
                reply_to: reply_to.clone(),
                media_ids: Vec::new(),
            };
            
            let result = self.post_text(post_content, &post_params)?;
            reply_to = Some(result.post_id.clone());
            results.push(result);
        }
        
        Ok(results)
    }
    
    fn upload_media(&self, media: &MediaUpload) -> ReedResult<String> {
        // Read file
        let file_content = std::fs::read(&media.path).map_err(|e| ReedError::IoError {
            operation: "read".to_string(),
            path: media.path.clone(),
            reason: e.to_string(),
        })?;
        
        // Upload to Mastodon
        let client = reqwest::blocking::Client::new();
        let form = reqwest::blocking::multipart::Form::new()
            .part("file", reqwest::blocking::multipart::Part::bytes(file_content));
        
        let response = client
            .post(self.api_url("media"))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .multipart(form)
            .send()
            .map_err(|e| ReedError::ExternalServiceError {
                service: "mastodon".to_string(),
                reason: e.to_string(),
            })?;
            
        let json: serde_json::Value = response.json().map_err(|e| ReedError::ExternalServiceError {
            service: "mastodon".to_string(),
            reason: format!("JSON parse error: {}", e),
        })?;
        
        Ok(json["id"].as_str().unwrap_or("").to_string())
    }
    
    fn test_connection(&self) -> ReedResult<bool> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(self.api_url("accounts/verify_credentials"))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .map_err(|e| ReedError::ExternalServiceError {
                service: "mastodon".to_string(),
                reason: e.to_string(),
            })?;
            
        Ok(response.status().is_success())
    }
}
```

## CLI Commands

```bash
# Add integration
reed integration:add mastodon --instance social.example.com --access-token xxx
reed integration:add twitter --api-key xxx --api-secret xxx --access-token xxx --access-secret xxx

# List integrations
reed integration:list

# Test integration
reed integration:test mastodon

# Post to social media (direct)
reed integration:post mastodon "Hello from ReedCMS!"
reed integration:post mastodon "Post with media" --media image.jpg

# Remove integration
reed integration:remove mastodon
```

## Hook Integration

Integrations are used by hooks and workflows:

**.reed/hooks.csv**
```csv
hook_id|trigger|condition|action|integration|parameters|status
blog-mastodon|after_set|key starts with 'blog.post'|post_to_integration|mastodon|visibility=public|active
```

## Testing Requirements

### Unit Tests
- [ ] API client implementations
- [ ] Thread posting logic
- [ ] Media upload
- [ ] Error handling

### Integration Tests (with mocks)
- [ ] Mastodon API
- [ ] Twitter API
- [ ] LinkedIn API
- [ ] Rate limiting
- [ ] Retry logic

## Acceptance Criteria
- [ ] Mastodon integration working
- [ ] Twitter integration working
- [ ] LinkedIn integration working
- [ ] CLI commands functional
- [ ] Hook integration working
- [ ] All tests pass
- [ ] BBC English throughout

## Security Considerations
- Access tokens encrypted in .reed/integrations.csv
- Rate limiting to avoid API bans
- Error logging (without exposing tokens)

## Future Extensions
- More platforms (Bluesky, Threads, etc.)
- Incoming webhook support (bidirectional)
- Content synchronisation (pull from social media)
- Analytics integration
