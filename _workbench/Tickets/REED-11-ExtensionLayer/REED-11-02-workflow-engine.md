# REED-11-02: Workflow Engine (Multi-Step Automation)

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files as `{name}_test.rs`

## Ticket Information
- **ID**: REED-11-02
- **Title**: Workflow Engine (Multi-Step Automation)
- **Layer**: Extension Layer (REED-11)
- **Priority**: Medium
- **Status**: Open
- **Complexity**: High
- **Dependencies**: REED-11-01 (Hook System), REED-04-10 (Agent Commands)

## Objective
Implement YAML-based workflow engine for complex multi-step automation with conditional execution, loops, and error handling.

## Use Case: Blog Publishing Workflow

**.reed/workflows/blog-publish.yml**
```yaml
name: blog-publish
description: Complete blog publishing workflow
trigger: manual  # or hook:after_create

steps:
  - name: validate_content
    action: agent:validate
    agent: claude
    input: ${context.content}
    parameters:
      check: grammar,tone,readability
    on_error: fail
    
  - name: generate_summary
    action: agent:generate
    agent: gpt4
    input: |
      Generate a 2-sentence summary:
      ${context.content}
    output: ${workflow.summary}
    
  - name: translate_content
    action: agent:translate
    agent: claude
    input: ${context.content}
    parameters:
      source_lang: de
      target_langs: [en, fr, es]
    output: ${workflow.translations}
    
  - name: save_translations
    action: loop
    items: ${workflow.translations}
    steps:
      - action: set:text
        key: ${context.base_key}@${item.language}
        value: ${item.content}
        
  - name: generate_social_post
    action: agent:generate
    agent: claude
    input: |
      Create Mastodon thread from:
      ${workflow.summary}
    output: ${workflow.social_thread}
    
  - name: post_to_mastodon
    action: post_to_mastodon
    input: ${workflow.social_thread}
    parameters:
      format: thread
      
  - name: notify_complete
    action: notify_email
    to: admin@example.com
    subject: Blog published
    body: |
      Blog post published successfully:
      - Translations: ${workflow.translations.length}
      - Posted to Mastodon
```

## Workflow Definition Format

### Basic Structure
```yaml
name: workflow_id
description: Workflow description
version: 1.0
trigger: manual | hook:trigger_name | schedule:cron_expression

variables:
  max_retries: 3
  timeout: 300
  
steps:
  - name: step_id
    action: action_type
    # ... action-specific fields
    
on_error:
  - action: notify_email
    to: admin@example.com
    
on_success:
  - action: log_event
    message: Workflow completed
```

### Supported Actions

**Agent Actions:**
- `agent:generate`: Generate content
- `agent:translate`: Translate content
- `agent:validate`: Validate content

**Reed Commands:**
- `set:text`: Set text content
- `set:route`: Set route
- `get:text`: Get text content

**External Actions:**
- `post_to_mastodon`: Post to Mastodon
- `post_to_twitter`: Post to Twitter
- `notify_email`: Send email
- `http_request`: HTTP API call

**Control Flow:**
- `loop`: Iterate over items
- `condition`: Conditional execution
- `parallel`: Parallel execution
- `wait`: Wait for duration/condition

### Variable System

**Context Variables:**
- `${context.key}`: Original trigger key
- `${context.value}`: Original trigger value
- `${context.user}`: User who triggered

**Workflow Variables:**
- `${workflow.step_name}`: Output from step
- `${workflow.summary}`: Persistent variable

**Environment Variables:**
- `${env.MASTODON_TOKEN}`: From .reed/server.csv

**System Variables:**
- `${timestamp}`: Current timestamp
- `${date}`: Current date

## Implementation

### File Structure
```
src/reedcms/
├── extensions/
│   ├── workflows/
│   │   ├── mod.rs
│   │   ├── engine.rs           # Workflow execution engine
│   │   ├── parser.rs           # YAML parser
│   │   ├── executor.rs         # Step executor
│   │   ├── variables.rs        # Variable resolution
│   │   └── actions/
│   │       ├── mod.rs
│   │       ├── agent.rs        # Agent actions
│   │       ├── reed.rs         # Reed command actions
│   │       ├── external.rs     # External API actions
│   │       └── control.rs      # Control flow actions
│   └── ...

.reed/workflows/                # Workflow definitions
├── blog-publish.yml
├── content-translation.yml
└── social-automation.yml
```

### Workflow Engine

```rust
// src/reedcms/extensions/workflows/engine.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub trigger: TriggerType,
    pub variables: Option<HashMap<String, serde_yaml::Value>>,
    pub steps: Vec<WorkflowStep>,
    pub on_error: Option<Vec<WorkflowStep>>,
    pub on_success: Option<Vec<WorkflowStep>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    Manual,
    Hook(String),
    Schedule(String),
}

#[derive(Debug, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub action: String,
    pub agent: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub parameters: Option<HashMap<String, serde_yaml::Value>>,
    pub on_error: Option<ErrorHandling>,
    pub condition: Option<String>,
    pub steps: Option<Vec<WorkflowStep>>,  // For nested steps
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandling {
    Fail,
    Continue,
    Retry,
}

/// Executes a workflow.
pub fn execute_workflow(
    workflow_id: &str,
    context: &WorkflowContext,
) -> ReedResult<WorkflowResult> {
    let workflow = load_workflow(workflow_id)?;
    let mut state = WorkflowState::new(context.clone());
    
    for step in &workflow.steps {
        // Check condition
        if let Some(ref condition) = step.condition {
            if !evaluate_condition(condition, &state)? {
                continue;
            }
        }
        
        // Execute step
        match execute_step(step, &mut state) {
            Ok(result) => {
                // Store output
                if let Some(ref output) = step.output {
                    state.set_variable(output, result)?;
                }
            }
            Err(e) => {
                // Handle error
                match step.on_error.as_ref().unwrap_or(&ErrorHandling::Fail) {
                    ErrorHandling::Fail => return Err(e),
                    ErrorHandling::Continue => continue,
                    ErrorHandling::Retry => {
                        // Retry logic
                        retry_step(step, &mut state, 3)?;
                    }
                }
            }
        }
    }
    
    // Execute on_success handlers
    if let Some(ref success_steps) = workflow.on_success {
        for step in success_steps {
            execute_step(step, &mut state)?;
        }
    }
    
    Ok(WorkflowResult {
        workflow_id: workflow.name,
        success: true,
        steps_executed: state.step_count,
        duration_ms: state.duration_ms(),
        output: state.export_variables(),
    })
}

#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub trigger: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub user: String,
}

pub struct WorkflowState {
    context: WorkflowContext,
    variables: HashMap<String, serde_json::Value>,
    step_count: usize,
    start_time: std::time::Instant,
}

impl WorkflowState {
    fn set_variable(&mut self, path: &str, value: serde_json::Value) -> ReedResult<()> {
        // Handle nested paths like ${workflow.summary}
        self.variables.insert(path.to_string(), value);
        Ok(())
    }
    
    fn resolve_variable(&self, var: &str) -> ReedResult<String> {
        // Resolve ${context.key}, ${workflow.step}, etc.
        // (Implementation...)
        Ok(String::new())
    }
}
```

### Step Execution

```rust
// src/reedcms/extensions/workflows/executor.rs

pub fn execute_step(
    step: &WorkflowStep,
    state: &mut WorkflowState,
) -> ReedResult<serde_json::Value> {
    // Resolve input variables
    let input = if let Some(ref input_template) = step.input {
        resolve_template(input_template, state)?
    } else {
        String::new()
    };
    
    // Execute based on action type
    match step.action.as_str() {
        "agent:generate" => {
            let agent_id = step.agent.as_ref().ok_or_else(|| ReedError::ValidationError {
                field: "agent".to_string(),
                value: String::new(),
                constraint: "Agent required for agent:generate action".to_string(),
            })?;
            
            execute_agent_generate(agent_id, &input, &step.parameters)
        }
        
        "agent:translate" => {
            execute_agent_translate(&step, &input, state)
        }
        
        "set:text" => {
            execute_set_text(&step, state)
        }
        
        "post_to_mastodon" => {
            execute_post_to_mastodon(&input, &step.parameters)
        }
        
        "loop" => {
            execute_loop(step, state)
        }
        
        _ => Err(ReedError::ValidationError {
            field: "action".to_string(),
            value: step.action.clone(),
            constraint: "Unknown action type".to_string(),
        }),
    }
}
```

## CLI Commands

```bash
# List workflows
reed workflow:list

# Show workflow definition
reed workflow:show blog-publish

# Run workflow
reed workflow:run blog-publish --key "blog.post.123" --value "content..."

# Run workflow with context
reed workflow:run blog-publish --context '{"key":"blog.post.123","content":"..."}'

# Validate workflow YAML
reed workflow:validate blog-publish.yml

# Create workflow from template
reed workflow:create my-workflow --template blog-publish
```

## Testing Requirements

### Unit Tests
- [ ] YAML parsing
- [ ] Variable resolution
- [ ] Condition evaluation
- [ ] Step execution
- [ ] Error handling

### Integration Tests
- [ ] Complete workflow execution
- [ ] Agent integration
- [ ] Loop execution
- [ ] Conditional execution
- [ ] Error recovery

## Acceptance Criteria
- [ ] YAML workflow parser working
- [ ] Variable resolution functional
- [ ] All action types implemented
- [ ] Conditional execution working
- [ ] Loop execution working
- [ ] Error handling robust
- [ ] CLI commands functional
- [ ] All tests pass
- [ ] BBC English throughout

## Dependencies
- REED-11-01: Hook system for trigger integration
- REED-04-10: Agent system for content operations
- External: serde_yaml for YAML parsing

## Future Extensions
- Visual workflow editor
- Workflow versioning
- Workflow marketplace/templates
- Real-time execution monitoring
