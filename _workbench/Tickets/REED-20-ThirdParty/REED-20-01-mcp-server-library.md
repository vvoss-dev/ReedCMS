# REED-20-01: MCP Server Library for Third-Party Integration

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files
- **Avoid**: Swiss Army knife functions

## Ticket Information
- **ID**: REED-20-01
- **Title**: MCP Server Library for Third-Party Integration
- **Layer**: Third-Party Integration (REED-20)
- **Priority**: Low
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-06-01 (Server Foundation), REED-07-01 (ReedAPI)

## Objective
Create a standalone MCP server library that allows external tools (Claude Desktop, IDEs, other MCP clients) to interact with ReedCMS instances. This enables bidirectional integration where ReedCMS can be both a client (REED-04-10, REED-11) and a server for external tools.

## Use Cases

### Use Case 1: Claude Desktop Integration
User in Claude Desktop can:
```
User: "List all blog posts in my ReedCMS"
Claude: [via MCP] reed list:text blog.post.*
Result: Shows list of blog posts

User: "Create a new blog post about Rust performance"
Claude: [via MCP] reed agent:generate blog.post.456.title@de --prompt "..." --agent claude
Result: Creates blog post with AI-generated content
```

### Use Case 2: VS Code Extension
Developer in VS Code can:
- Browse ReedCMS content in sidebar
- Edit text/routes/meta directly
- Preview layouts
- Run migrations
- Test integrations

### Use Case 3: CI/CD Pipeline
GitHub Actions can:
- Validate content before merge
- Auto-deploy content updates
- Run scheduled content generation
- Sync with external systems

## MCP Protocol Overview

**MCP (Model Context Protocol)** by Anthropic:
- Standard protocol for AI tools to access external resources
- JSON-RPC based communication
- Server provides "tools" (callable functions)
- Server provides "resources" (readable data)
- Server can send "prompts" (suggested actions)

## Architecture

### Package Structure
```
reed-mcp-server/                    # Separate crate/package
├── Cargo.toml                      # Rust crate
├── src/
│   ├── lib.rs                      # Library exports
│   ├── server.rs                   # MCP server implementation
│   ├── tools/                      # MCP tools (callable functions)
│   │   ├── mod.rs
│   │   ├── data.rs                 # set:text, get:text, list:text
│   │   ├── layout.rs               # init:layout
│   │   ├── user.rs                 # user:create, user:list
│   │   ├── agent.rs                # agent:generate, agent:translate
│   │   └── migration.rs            # migrate:text, validate:routes
│   ├── resources/                  # MCP resources (readable data)
│   │   ├── mod.rs
│   │   ├── content.rs              # Content browser
│   │   └── schema.rs               # Schema information
│   └── bridge.rs                   # ReedCMS integration bridge
├── examples/
│   └── standalone_server.rs        # Example usage
└── README.md
```

### Separate from ReedCMS Core
**Important**: This is a **separate package** that **imports** ReedCMS:

```toml
# reed-mcp-server/Cargo.toml
[package]
name = "reed-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
reedcms = { path = "../", version = "0.1.0" }  # Import ReedCMS
mcp-server-sdk = "0.1"  # Hypothetical MCP SDK
serde = "1.0"
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

## MCP Tools (Callable Functions)

### Data Tools
```json
{
  "name": "reed_set_text",
  "description": "Set text content in ReedCMS",
  "inputSchema": {
    "type": "object",
    "properties": {
      "key": { "type": "string", "description": "Key with @lang suffix" },
      "value": { "type": "string", "description": "Text content" },
      "description": { "type": "string", "description": "Optional description" }
    },
    "required": ["key", "value"]
  }
}
```

```json
{
  "name": "reed_get_text",
  "description": "Get text content from ReedCMS",
  "inputSchema": {
    "type": "object",
    "properties": {
      "key": { "type": "string", "description": "Key to retrieve" }
    },
    "required": ["key"]
  }
}
```

```json
{
  "name": "reed_list_text",
  "description": "List text entries matching pattern",
  "inputSchema": {
    "type": "object",
    "properties": {
      "pattern": { "type": "string", "description": "Key pattern (supports wildcards)" }
    }
  }
}
```

### Agent Tools
```json
{
  "name": "reed_agent_generate",
  "description": "Generate content using AI agent",
  "inputSchema": {
    "type": "object",
    "properties": {
      "output_key": { "type": "string" },
      "prompt": { "type": "string" },
      "agent": { "type": "string", "description": "Agent ID" },
      "max_tokens": { "type": "number" }
    },
    "required": ["output_key", "prompt", "agent"]
  }
}
```

### Migration Tools
```json
{
  "name": "reed_migrate_text",
  "description": "Migrate text from CSV files",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "recursive": { "type": "boolean" },
      "dry_run": { "type": "boolean" }
    },
    "required": ["path"]
  }
}
```

## MCP Resources (Readable Data)

### Content Browser Resource
```json
{
  "uri": "reed://content/text",
  "name": "ReedCMS Text Content",
  "description": "Browse all text content",
  "mimeType": "application/json"
}
```

Response:
```json
{
  "entries": [
    {
      "key": "blog.post.123.title@de",
      "value": "Rust Performance Tricks",
      "description": "Blog post title"
    }
  ]
}
```

### Schema Resource
```json
{
  "uri": "reed://schema",
  "name": "ReedCMS Schema",
  "description": "Database schema information",
  "mimeType": "application/json"
}
```

## Implementation

### MCP Server Core

```rust
// reed-mcp-server/src/server.rs

use mcp_server_sdk::{Server, Tool, Resource};
use reedcms::cli;  // Import ReedCMS CLI functions
use serde_json::json;

pub struct ReedMcpServer {
    reed_root: PathBuf,
}

impl ReedMcpServer {
    pub fn new(reed_root: PathBuf) -> Self {
        Self { reed_root }
    }
    
    pub async fn start(&self, transport: Transport) -> Result<()> {
        let mut server = Server::new();
        
        // Register tools
        server.register_tool(Tool {
            name: "reed_set_text".to_string(),
            description: "Set text content in ReedCMS".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "value": { "type": "string" },
                    "description": { "type": "string" }
                },
                "required": ["key", "value"]
            }),
            handler: Box::new(|params| {
                self.handle_set_text(params)
            }),
        });
        
        server.register_tool(Tool {
            name: "reed_get_text".to_string(),
            description: "Get text content from ReedCMS".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" }
                },
                "required": ["key"]
            }),
            handler: Box::new(|params| {
                self.handle_get_text(params)
            }),
        });
        
        // Register more tools...
        
        // Register resources
        server.register_resource(Resource {
            uri: "reed://content/text".to_string(),
            name: "ReedCMS Text Content".to_string(),
            description: "Browse all text content".to_string(),
            mime_type: "application/json".to_string(),
            handler: Box::new(|| {
                self.handle_text_resource()
            }),
        });
        
        // Start server
        server.run(transport).await
    }
    
    fn handle_set_text(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let key = params["key"].as_str().unwrap();
        let value = params["value"].as_str().unwrap();
        let description = params["description"].as_str();
        
        // Change to ReedCMS directory
        std::env::set_current_dir(&self.reed_root)?;
        
        // Execute ReedCMS command
        let mut flags = HashMap::new();
        if let Some(desc) = description {
            flags.insert("description".to_string(), desc.to_string());
        }
        
        let response = cli::data_commands::set_text(
            &[key.to_string(), value.to_string()],
            &flags
        )?;
        
        Ok(json!({
            "success": true,
            "message": response.data
        }))
    }
    
    fn handle_get_text(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let key = params["key"].as_str().unwrap();
        
        std::env::set_current_dir(&self.reed_root)?;
        
        let response = cli::data_commands::get_text(&[key.to_string()])?;
        
        Ok(json!({
            "key": key,
            "value": response.data
        }))
    }
    
    fn handle_text_resource(&self) -> Result<serde_json::Value> {
        std::env::set_current_dir(&self.reed_root)?;
        
        let response = cli::data_commands::list_text(&[])?;
        
        // Parse response and format as JSON
        Ok(json!({
            "entries": parse_text_list(&response.data)
        }))
    }
}
```

### Standalone Server Binary

```rust
// reed-mcp-server/examples/standalone_server.rs

use reed_mcp_server::ReedMcpServer;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Get ReedCMS root from environment or argument
    let reed_root = std::env::var("REED_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    
    println!("Starting ReedCMS MCP Server");
    println!("ReedCMS root: {}", reed_root.display());
    
    let server = ReedMcpServer::new(reed_root);
    
    // Use stdio transport (standard for MCP)
    let transport = mcp_server_sdk::StdioTransport::new();
    
    server.start(transport).await?;
    
    Ok(())
}
```

## Claude Desktop Configuration

Users can add ReedCMS to Claude Desktop:

**~/.config/Claude/claude_desktop_config.json** (Linux/macOS)
```json
{
  "mcpServers": {
    "reedcms": {
      "command": "reed-mcp-server",
      "args": [],
      "env": {
        "REED_ROOT": "/path/to/my/reedcms/project"
      }
    }
  }
}
```

**%APPDATA%\Claude\claude_desktop_config.json** (Windows)
```json
{
  "mcpServers": {
    "reedcms": {
      "command": "reed-mcp-server.exe",
      "args": [],
      "env": {
        "REED_ROOT": "C:\\path\\to\\my\\reedcms\\project"
      }
    }
  }
}
```

## Distribution

### 1. Crates.io (Rust Package)
```bash
cargo install reed-mcp-server
```

### 2. npm Package (for Node.js users)
```bash
npm install -g @reedcms/mcp-server
```

### 3. Homebrew (macOS)
```bash
brew install reedcms/tap/reed-mcp-server
```

### 4. MCP Server Directory
List on official Anthropic MCP server directory:
- https://github.com/anthropics/mcp-servers
- https://modelcontextprotocol.io/servers

## Testing Requirements

### Unit Tests
- [ ] Tool handler functions
- [ ] Resource handlers
- [ ] Parameter validation
- [ ] Error handling

### Integration Tests
- [ ] Full MCP protocol communication
- [ ] ReedCMS command execution
- [ ] Resource browsing
- [ ] Authentication (if needed)

### Manual Tests
- [ ] Claude Desktop integration
- [ ] VS Code extension integration
- [ ] Standalone server mode

## Acceptance Criteria
- [ ] MCP server library functional
- [ ] All ReedCMS commands exposed as tools
- [ ] Content browsable as resources
- [ ] Standalone binary working
- [ ] Claude Desktop integration verified
- [ ] Published to crates.io
- [ ] Listed in MCP server directory
- [ ] Documentation complete
- [ ] BBC English throughout

## Documentation Requirements

### README.md
- Installation instructions
- Configuration guide (Claude Desktop, VS Code)
- Available tools and resources
- Example usage
- Troubleshooting

### API Documentation
- All tools with input/output schemas
- All resources with URIs
- Authentication (if applicable)
- Rate limiting (if applicable)

## Security Considerations

### Authentication
- Consider adding API key authentication
- Read from .reed/server.csv
- Optional: per-tool permissions

### Sandboxing
- Limit which commands can be executed
- Whitelist/blacklist mode
- Read-only mode option

## Future Extensions

### REED-20-02: VS Code Extension
- Tree view for content
- Direct editing
- Preview layouts
- Integrated terminal

### REED-20-03: GitHub Actions Integration
- Pre-built actions for CI/CD
- Content validation
- Automated deployments

### REED-20-04: Zapier/Make.com Integration
- ReedCMS as Zapier app
- Triggers and actions
- No-code automation

## Notes
- Keep as separate package to avoid bloating core ReedCMS
- Follow MCP specification strictly for compatibility
- Ensure backwards compatibility with ReedCMS versions
- Clear versioning strategy (semver)
