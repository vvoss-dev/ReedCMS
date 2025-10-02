# REED-04-10: CLI Agent Commands

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}_test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `middleware.rs`, `utils.rs`

## Ticket Information
- **ID**: REED-04-10
- **Title**: CLI Agent Commands (MCP Integration Foundation)
- **Layer**: CLI Layer (REED-04)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-04-01, REED-02-05 (Matrix CSV)

## Summary Reference
- **Section**: CLI Agent Management
- **Key Concepts**: MCP provider integration, API key management, content generation, bidirectional automation foundation

## Objective
Implement CLI commands for managing AI agents (MCP providers) and basic content generation capabilities. This provides the foundation for bidirectional automation (REED-11) while keeping direct CLI usage simple and practical.

## Requirements

### Agent Management Commands

```bash
# Add new agent
reed agent:add <agent_id> --provider <provider> --api-key <key> --model <model> [flags]
reed agent:add claude --provider anthropic --api-key sk-xxx --model claude-sonnet-4
reed agent:add gpt4 --provider openai --api-key sk-xxx --model gpt-4-turbo --temperature 0.8

# List agents
reed agent:list [--format table|json|csv]
reed agent:list --format json

# Show agent details
reed agent:show <agent_id>
reed agent:show claude

# Test agent connection
reed agent:test <agent_id> [--prompt "test message"]
reed agent:test claude --prompt "Say hello"

# Update agent configuration
reed agent:update <agent_id> [--model <model>] [--temperature <temp>] [--max-tokens <tokens>]
reed agent:update claude --model claude-opus-4 --temperature 0.9

# Remove agent
reed agent:remove <agent_id> [--force]
reed agent:remove old-agent --force
```

### Content Generation Commands

```bash
# Generate text content
reed agent:generate <output_key> --prompt <prompt> --agent <agent_id> [flags]
reed agent:generate blog.post.123.title@de --prompt "Title for blog about Rust performance" --agent claude
reed agent:generate knowledge.intro.text@en --prompt "Introduction to ReedCMS" --agent gpt4 --max-tokens 500

# Translate content
reed agent:translate <source_key> --to <lang>[,<lang>...] --agent <agent_id>
reed agent:translate blog.post.123.title@de --to en,fr --agent claude
reed agent:translate knowledge.intro@de --to en,es,fr,it --agent gpt4

# Batch generation
reed agent:batch <file> --agent <agent_id>
reed agent:batch prompts.csv --agent claude
```

### Data Storage

**.reed/agents.matrix.csv**
```csv
agent_id|provider|api_key_encrypted|model|max_tokens|temperature|top_p|status|created_by|created_at|updated_at
claude|anthropic|[encrypted-base64]|claude-sonnet-4|4096|0.7|0.95|active|admin|2025-10-02T...|2025-10-02T...
gpt4|openai|[encrypted-base64]|gpt-4-turbo|8192|0.8|1.0|active|admin|2025-10-02T...|2025-10-02T...
```

**Fields:**
- `agent_id`: Unique identifier (alphanumeric + hyphens)
- `provider`: anthropic, openai, custom
- `api_key_encrypted`: Encrypted with system key from .reed/server.csv
- `model`: Model identifier
- `max_tokens`: Maximum tokens per request
- `temperature`: 0.0-2.0 (creativity)
- `top_p`: 0.0-1.0 (nucleus sampling)
- `status`: active, inactive, error
- `created_by`: User who added agent
- `created_at`, `updated_at`: Timestamps

## Implementation

### File Structure

```
src/reedcms/
├── cli/
│   ├── agent_commands.rs        # Agent management commands
│   └── agent_commands_test.rs   # Tests
│
├── agents/                       # NEW module
│   ├── mod.rs                   # Public exports
│   ├── registry.rs              # Agent registry management
│   ├── encryption.rs            # API key encryption/decryption
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── anthropic.rs         # Anthropic Claude MCP
│   │   ├── openai.rs            # OpenAI MCP
│   │   └── traits.rs            # Provider trait
│   ├── generation.rs            # Content generation logic
│   └── translation.rs           # Translation logic
```

### Agent Provider Trait

```rust
// src/reedcms/agents/providers/traits.rs

use crate::reedcms::reedstream::{ReedResult, ReedResponse};

/// Agent provider trait for MCP integration.
pub trait AgentProvider {
    /// Generate text completion.
    fn generate(&self, prompt: &str, config: &GenerationConfig) -> ReedResult<String>;
    
    /// Test connection and API key.
    fn test_connection(&self) -> ReedResult<bool>;
    
    /// Get provider name.
    fn provider_name(&self) -> &str;
    
    /// Get model name.
    fn model_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub stop_sequences: Vec<String>,
}
```

### Anthropic Provider Implementation

```rust
// src/reedcms/agents/providers/anthropic.rs

use super::traits::{AgentProvider, GenerationConfig};
use crate::reedcms::reedstream::{ReedResult, ReedError};

pub struct AnthropicProvider {
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl AgentProvider for AnthropicProvider {
    fn generate(&self, prompt: &str, config: &GenerationConfig) -> ReedResult<String> {
        // HTTP request to Anthropic API
        // POST https://api.anthropic.com/v1/messages
        // Headers: x-api-key, anthropic-version: 2023-06-01
        // Body: { model, messages: [{ role: "user", content: prompt }], max_tokens, temperature }
        
        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": self.model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": config.max_tokens,
                "temperature": config.temperature,
            }))
            .send()
            .map_err(|e| ReedError::ExternalServiceError {
                service: "anthropic".to_string(),
                reason: e.to_string(),
            })?;
            
        if !response.status().is_success() {
            return Err(ReedError::ExternalServiceError {
                service: "anthropic".to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }
        
        let json: serde_json::Value = response.json().map_err(|e| ReedError::ExternalServiceError {
            service: "anthropic".to_string(),
            reason: format!("JSON parse error: {}", e),
        })?;
        
        // Extract text from response
        let text = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| ReedError::ExternalServiceError {
                service: "anthropic".to_string(),
                reason: "Missing text in response".to_string(),
            })?;
            
        Ok(text.to_string())
    }
    
    fn test_connection(&self) -> ReedResult<bool> {
        self.generate("test", &GenerationConfig {
            max_tokens: 10,
            temperature: 0.7,
            top_p: 0.95,
            stop_sequences: Vec::new(),
        })?;
        Ok(true)
    }
    
    fn provider_name(&self) -> &str {
        "anthropic"
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
}
```

### API Key Encryption

```rust
// src/reedcms/agents/encryption.rs

use crate::reedcms::csv::read_csv;
use crate::reedcms::reedstream::{ReedResult, ReedError};
use base64::{Engine as _, engine::general_purpose};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::path::PathBuf;

/// Encrypts API key using system key from .reed/server.csv.
pub fn encrypt_api_key(api_key: &str) -> ReedResult<String> {
    let system_key = get_system_key()?;
    let cipher = Aes256Gcm::new_from_slice(system_key.as_bytes())
        .map_err(|e| ReedError::ConfigError {
            component: "encryption".to_string(),
            reason: format!("Invalid key length: {}", e),
        })?;
        
    let nonce = Nonce::from_slice(b"unique nonce"); // Use proper random nonce in production
    let ciphertext = cipher
        .encrypt(nonce, api_key.as_bytes())
        .map_err(|e| ReedError::ConfigError {
            component: "encryption".to_string(),
            reason: format!("Encryption failed: {}", e),
        })?;
        
    Ok(general_purpose::STANDARD.encode(ciphertext))
}

/// Decrypts API key using system key.
pub fn decrypt_api_key(encrypted: &str) -> ReedResult<String> {
    let system_key = get_system_key()?;
    let cipher = Aes256Gcm::new_from_slice(system_key.as_bytes())
        .map_err(|e| ReedError::ConfigError {
            component: "encryption".to_string(),
            reason: format!("Invalid key length: {}", e),
        })?;
        
    let ciphertext = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| ReedError::ConfigError {
            component: "encryption".to_string(),
            reason: format!("Base64 decode failed: {}", e),
        })?;
        
    let nonce = Nonce::from_slice(b"unique nonce");
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| ReedError::ConfigError {
            component: "encryption".to_string(),
            reason: format!("Decryption failed: {}", e),
        })?;
        
    String::from_utf8(plaintext).map_err(|e| ReedError::ConfigError {
        component: "encryption".to_string(),
        reason: format!("UTF-8 decode failed: {}", e),
    })
}

/// Retrieves system encryption key from .reed/server.csv.
fn get_system_key() -> ReedResult<String> {
    let server_path = PathBuf::from(".reed/server.csv");
    if !server_path.exists() {
        return Err(ReedError::NotFound {
            resource: "server.csv".to_string(),
            context: Some("System encryption key not configured".to_string()),
        });
    }
    
    let entries = read_csv(&server_path)?;
    let key_entry = entries
        .iter()
        .find(|e| e.key == "encryption.system_key")
        .ok_or_else(|| ReedError::NotFound {
            resource: "encryption.system_key".to_string(),
            context: Some("System key not found in server.csv".to_string()),
        })?;
        
    Ok(key_entry.value.clone())
}
```

### CLI Commands Implementation

```rust
// src/reedcms/cli/agent_commands.rs

use crate::reedcms::agents::{
    add_agent, list_agents, get_agent, test_agent, update_agent, remove_agent,
    generate_content, translate_content, AgentConfig,
};
use crate::reedcms::reedstream::{ReedError, ReedResponse, ReedResult};
use std::collections::HashMap;

/// Adds a new agent to the registry.
///
/// ## Usage
/// reed agent:add <agent_id> --provider <provider> --api-key <key> --model <model> [flags]
///
/// ## Required Flags
/// - --provider: anthropic, openai
/// - --api-key: API key for provider
/// - --model: Model identifier
///
/// ## Optional Flags
/// - --max-tokens: Maximum tokens (default: 4096)
/// - --temperature: Temperature 0.0-2.0 (default: 0.7)
/// - --top-p: Top-p sampling 0.0-1.0 (default: 0.95)
///
/// ## Example
/// ```bash
/// reed agent:add claude --provider anthropic --api-key sk-xxx --model claude-sonnet-4
/// ```
pub fn add_agent(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "agent_id".to_string(),
            value: String::new(),
            constraint: "agent_id required".to_string(),
        });
    }
    
    let agent_id = &args[0];
    
    let provider = flags.get("provider").ok_or_else(|| ReedError::ValidationError {
        field: "provider".to_string(),
        value: String::new(),
        constraint: "--provider flag required".to_string(),
    })?;
    
    let api_key = flags.get("api-key").ok_or_else(|| ReedError::ValidationError {
        field: "api-key".to_string(),
        value: String::new(),
        constraint: "--api-key flag required".to_string(),
    })?;
    
    let model = flags.get("model").ok_or_else(|| ReedError::ValidationError {
        field: "model".to_string(),
        value: String::new(),
        constraint: "--model flag required".to_string(),
    })?;
    
    let config = AgentConfig {
        provider: provider.clone(),
        model: model.clone(),
        max_tokens: flags
            .get("max-tokens")
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096),
        temperature: flags
            .get("temperature")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.7),
        top_p: flags
            .get("top-p")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.95),
    };
    
    let response = add_agent_service(agent_id, api_key, config)?;
    
    let output = format!(
        "Agent added: {}\nProvider: {}\nModel: {}\nStatus: active",
        agent_id, provider, model
    );
    
    Ok(ReedResponse {
        data: output,
        source: response.source,
        cached: false,
        timestamp: response.timestamp,
        metrics: response.metrics,
    })
}

/// Generates content using an agent.
///
/// ## Usage
/// reed agent:generate <output_key> --prompt <prompt> --agent <agent_id> [flags]
///
/// ## Required Flags
/// - --prompt: Generation prompt
/// - --agent: Agent ID to use
///
/// ## Optional Flags
/// - --max-tokens: Override agent's default
/// - --temperature: Override agent's default
/// - --dry-run: Preview without saving
///
/// ## Example
/// ```bash
/// reed agent:generate blog.post.123.title@de --prompt "Title about Rust" --agent claude
/// ```
pub fn generate(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    if args.is_empty() {
        return Err(ReedError::ValidationError {
            field: "output_key".to_string(),
            value: String::new(),
            constraint: "output key required".to_string(),
        });
    }
    
    let output_key = &args[0];
    let prompt = flags.get("prompt").ok_or_else(|| ReedError::ValidationError {
        field: "prompt".to_string(),
        value: String::new(),
        constraint: "--prompt flag required".to_string(),
    })?;
    
    let agent_id = flags.get("agent").ok_or_else(|| ReedError::ValidationError {
        field: "agent".to_string(),
        value: String::new(),
        constraint: "--agent flag required".to_string(),
    })?;
    
    let dry_run = flags.contains_key("dry-run");
    
    let response = generate_content_service(agent_id, prompt, output_key, dry_run)?;
    
    Ok(response)
}
```

## Security Considerations

### API Key Storage
- **NEVER** store plaintext API keys
- Encrypt with AES-256-GCM using system key from .reed/server.csv
- System key must be 32 bytes (256 bits)
- Use proper random nonces (not hardcoded)

### System Key Generation
```bash
# Initialize system encryption key
reed init:security --generate-encryption-key
```

This creates entry in .reed/server.csv:
```csv
encryption.system_key|[32-byte-base64-key]|System encryption key for API keys
```

### Rate Limiting
- Track API calls per agent
- Warn if approaching provider limits
- Store in .reed/agents.usage.csv

## Testing Requirements

### Unit Tests
- [ ] Agent add/remove/update
- [ ] Encryption/decryption
- [ ] Provider trait implementations
- [ ] Content generation
- [ ] Translation logic

### Integration Tests
- [ ] Full agent lifecycle
- [ ] API calls to providers (with test keys)
- [ ] Error handling

### Security Tests
- [ ] API key encryption strength
- [ ] Key never appears in logs
- [ ] Decryption failures handled

## Dependencies

**Cargo.toml additions:**
```toml
[dependencies]
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde_json = "1.0"
aes-gcm = "0.10"
base64 = "0.21"
```

## Acceptance Criteria
- [ ] Agent add/list/show/test/update/remove working
- [ ] API key encryption implemented
- [ ] Anthropic provider working
- [ ] OpenAI provider working
- [ ] Content generation functional
- [ ] Translation functional
- [ ] All tests pass
- [ ] Documentation complete
- [ ] BBC English throughout

## Future Extensions (REED-11)
This ticket provides foundation for:
- REED-11-01: Hook System (trigger-based automation)
- REED-11-02: Workflow Engine (complex automation)
- REED-11-03: External Integrations (social media, etc.)

## Notes
- Keep CLI interface simple and practical
- Complex automation belongs in REED-11
- Focus on direct, manual usage first
- Encryption is critical - no shortcuts
- Test with actual API calls (use test keys)
