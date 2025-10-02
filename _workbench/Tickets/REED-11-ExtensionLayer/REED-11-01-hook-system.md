# REED-11-01: Hook System (Trigger-Based Automation)

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files as `{name}_test.rs`
- **Avoid**: Swiss Army knife functions
- **Avoid**: Generic file names

## Ticket Information
- **ID**: REED-11-01
- **Title**: Hook System (Trigger-Based Automation)
- **Layer**: Extension Layer (REED-11)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-04-10 (CLI Agent Commands)

## Objective
Implement event-driven hook system that automatically executes actions when specific triggers fire. Enables bidirectional automation like auto-posting to social media after content creation.

## Use Cases

### Example 1: Blog Post to Social Media
```csv
hook_id|trigger|condition|action|agent|parameters|status
blog-social|after_set|key starts with 'blog.post'|post_to_mastodon|claude|format=thread,max_length=500|active
```

**Flow:**
1. User: `reed set:text blog.post.123.content@de "Rust Performance..."`
2. Trigger fires: `after_set` with key `blog.post.123.content@de`
3. Condition matches: key starts with `blog.post`
4. Action executes: `post_to_mastodon` using `claude` agent
5. Agent formats content as Mastodon thread
6. Posts to configured Mastodon account

### Example 2: Auto-Translation
```csv
hook_id|trigger|condition|action|agent|parameters|status
auto-translate|after_create|layout=knowledge|translate_content|gpt4|target_langs=en,fr|active
```

### Example 3: Content Validation
```csv
hook_id|trigger|condition|action|agent|parameters|status
validate-text|before_set|key starts with 'blog'|validate_content|claude|check=grammar,tone|active
```

## Data Structure

**.reed/hooks.csv**
```csv
hook_id|trigger|condition|action|agent|parameters|status|priority|created_by|created_at
blog-social|after_set|key starts with 'blog.post'|post_to_mastodon|claude|format=thread|active|100|admin|2025-10-02T...
translate|after_create|layout=knowledge|translate_content|gpt4|target_langs=de,en,fr|active|50|admin|2025-10-02T...
```

**Fields:**
- `hook_id`: Unique identifier
- `trigger`: Event that fires hook (after_set, after_create, before_set, etc.)
- `condition`: Match condition (key pattern, layout, language, etc.)
- `action`: Action to execute
- `agent`: Agent ID to use (from REED-04-10)
- `parameters`: JSON/CSV parameters for action
- `status`: active, inactive, error
- `priority`: Execution order (higher = first)
- `created_by`, `created_at`: Audit trail

## Triggers

### Data Triggers
- `after_set`: After `reed set:text/route/meta`
- `before_set`: Before `reed set:text/route/meta`
- `after_create`: After layout/user/role creation
- `after_update`: After update operation
- `after_delete`: After deletion

### Time Triggers (REED-11-04)
- `schedule`: Cron-style scheduled execution
- `interval`: Regular interval execution

## Actions

### Content Actions
- `post_to_mastodon`: Post to Mastodon
- `post_to_twitter`: Post to Twitter/X
- `post_to_linkedin`: Post to LinkedIn
- `translate_content`: Auto-translate to languages
- `generate_summary`: Generate content summary
- `validate_content`: Content validation

### System Actions
- `notify_email`: Send email notification
- `log_event`: Log to file
- `backup_content`: Create backup
- `run_workflow`: Execute workflow (REED-11-02)

## Implementation

### File Structure
```
src/reedcms/
├── extensions/                    # NEW module
│   ├── mod.rs
│   ├── hooks/
│   │   ├── mod.rs
│   │   ├── registry.rs           # Hook registration/storage
│   │   ├── dispatcher.rs         # Hook execution engine
│   │   ├── triggers.rs           # Trigger definitions
│   │   ├── conditions.rs         # Condition matching
│   │   └── actions/
│   │       ├── mod.rs
│   │       ├── social.rs         # Social media posting
│   │       ├── content.rs        # Content operations
│   │       └── system.rs         # System operations
│   └── ...
```

### Hook Dispatcher

```rust
// src/reedcms/extensions/hooks/dispatcher.rs

use crate::reedcms::reedstream::{ReedResult, ReedError};

/// Hook context passed to actions.
#[derive(Debug, Clone)]
pub struct HookContext {
    pub trigger: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub layout: Option<String>,
    pub language: Option<String>,
    pub user: String,
}

/// Dispatches hooks for a trigger event.
///
/// ## Flow
/// 1. Load active hooks for trigger
/// 2. Sort by priority (descending)
/// 3. For each hook:
///    - Check condition match
///    - Execute action
///    - Log result
///
/// ## Example
/// ```rust
/// let context = HookContext {
///     trigger: "after_set".to_string(),
///     key: Some("blog.post.123.title@de".to_string()),
///     value: Some("My Blog Post".to_string()),
///     layout: None,
///     language: Some("de".to_string()),
///     user: "admin".to_string(),
/// };
/// dispatch_hooks(&context)?;
/// ```
pub fn dispatch_hooks(context: &HookContext) -> ReedResult<Vec<HookResult>> {
    let hooks = load_hooks_for_trigger(&context.trigger)?;
    let mut results = Vec::new();
    
    for hook in hooks {
        // Check condition
        if !check_condition(&hook.condition, context)? {
            continue;
        }
        
        // Execute action
        let result = execute_action(&hook, context)?;
        results.push(result);
        
        // Log execution
        log_hook_execution(&hook, &result)?;
    }
    
    Ok(results)
}

#[derive(Debug)]
pub struct HookResult {
    pub hook_id: String,
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
}
```

### Condition Matching

```rust
// src/reedcms/extensions/hooks/conditions.rs

/// Checks if condition matches context.
///
/// ## Supported Conditions
/// - `key starts with 'blog'`
/// - `key ends with '@de'`
/// - `key contains 'post'`
/// - `layout = knowledge`
/// - `language = de`
/// - `always` (always matches)
///
/// ## Example
/// ```rust
/// let context = HookContext { key: Some("blog.post.123@de"), ... };
/// assert!(check_condition("key starts with 'blog'", &context)?);
/// ```
pub fn check_condition(condition: &str, context: &HookContext) -> ReedResult<bool> {
    // Parse condition
    let parts: Vec<&str> = condition.split_whitespace().collect();
    
    match parts.as_slice() {
        ["always"] => Ok(true),
        
        ["key", "starts", "with", pattern] => {
            if let Some(ref key) = context.key {
                let pattern = pattern.trim_matches('\'').trim_matches('"');
                Ok(key.starts_with(pattern))
            } else {
                Ok(false)
            }
        }
        
        ["key", "ends", "with", pattern] => {
            if let Some(ref key) = context.key {
                let pattern = pattern.trim_matches('\'').trim_matches('"');
                Ok(key.ends_with(pattern))
            } else {
                Ok(false)
            }
        }
        
        ["key", "contains", pattern] => {
            if let Some(ref key) = context.key {
                let pattern = pattern.trim_matches('\'').trim_matches('"');
                Ok(key.contains(pattern))
            } else {
                Ok(false)
            }
        }
        
        ["layout", "=", layout] => {
            if let Some(ref ctx_layout) = context.layout {
                Ok(ctx_layout == layout)
            } else {
                Ok(false)
            }
        }
        
        ["language", "=", lang] => {
            if let Some(ref ctx_lang) = context.language {
                Ok(ctx_lang == lang)
            } else {
                Ok(false)
            }
        }
        
        _ => Err(ReedError::ValidationError {
            field: "condition".to_string(),
            value: condition.to_string(),
            constraint: "Unknown condition format".to_string(),
        }),
    }
}
```

### Social Media Action

```rust
// src/reedcms/extensions/hooks/actions/social.rs

use crate::reedcms::agents::get_agent;
use crate::reedcms::extensions::hooks::HookContext;

/// Posts content to Mastodon.
///
/// ## Parameters
/// - format: thread, single (default: single)
/// - max_length: Maximum post length (default: 500)
///
/// ## Example
/// ```rust
/// let params = "format=thread,max_length=500";
/// post_to_mastodon(&hook, &context, params)?;
/// ```
pub fn post_to_mastodon(
    hook: &Hook,
    context: &HookContext,
    parameters: &str,
) -> ReedResult<String> {
    // Parse parameters
    let params = parse_parameters(parameters)?;
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("single");
    let max_length: usize = params
        .get("max_length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    
    // Get content
    let content = context.value.as_ref().ok_or_else(|| ReedError::ValidationError {
        field: "value".to_string(),
        value: String::new(),
        constraint: "Content required for posting".to_string(),
    })?;
    
    // Format content using agent
    let agent = get_agent(&hook.agent)?;
    let formatted = if format == "thread" {
        format_as_thread(agent, content, max_length)?
    } else {
        format_as_single(agent, content, max_length)?
    };
    
    // Get Mastodon credentials from .reed/integrations.csv
    let credentials = get_mastodon_credentials()?;
    
    // Post to Mastodon API
    post_to_mastodon_api(&credentials, &formatted)?;
    
    Ok(format!("Posted to Mastodon: {} character(s)", formatted.len()))
}

fn format_as_thread(agent: &Agent, content: &str, max_length: usize) -> ReedResult<Vec<String>> {
    let prompt = format!(
        "Split this content into a Mastodon thread. Each post max {} chars:\n\n{}",
        max_length, content
    );
    
    let response = agent.generate(&prompt)?;
    
    // Parse response into thread posts
    // (Implementation details...)
    
    Ok(vec![response])
}
```

## CLI Integration

Hook management via existing CLI or new commands:

```bash
# List hooks
reed hook:list [--trigger <trigger>] [--status active|inactive]

# Add hook
reed hook:add <hook_id> --trigger <trigger> --condition <condition> --action <action> --agent <agent>

# Enable/disable hook
reed hook:enable <hook_id>
reed hook:disable <hook_id>

# Test hook manually
reed hook:test <hook_id> --key "blog.post.123@de" --value "test content"

# Show hook execution log
reed hook:log <hook_id> [--last <n>]
```

## Integration Points

### In CLI Commands
Modify existing commands to fire hooks:

```rust
// In src/reedcms/cli/data_commands.rs

pub fn set_text(args: &[String], flags: &HashMap<String, String>) -> ReedResult<ReedResponse<String>> {
    // ... existing code ...
    
    // Fire after_set hook
    let context = HookContext {
        trigger: "after_set".to_string(),
        key: Some(key.clone()),
        value: Some(value.clone()),
        layout: None,
        language: extract_language(&key),
        user: "system".to_string(),
    };
    
    dispatch_hooks(&context)?;
    
    Ok(response)
}
```

## Testing Requirements

### Unit Tests
- [ ] Condition matching logic
- [ ] Hook execution order (priority)
- [ ] Parameter parsing
- [ ] Action execution

### Integration Tests
- [ ] End-to-end hook firing
- [ ] Social media posting (with mock API)
- [ ] Agent integration
- [ ] Error handling

## Acceptance Criteria
- [ ] Hook registry working
- [ ] Trigger dispatching functional
- [ ] Condition matching accurate
- [ ] Actions execute correctly
- [ ] Social media integration working
- [ ] CLI commands functional
- [ ] All tests pass
- [ ] BBC English throughout

## Dependencies
- REED-04-10: Agent system for content processing
- External: Mastodon API, Twitter API credentials

## Future Extensions
- REED-11-02: Workflow engine builds on hooks
- REED-11-03: More external API bridges
- REED-11-04: Scheduled hook execution
