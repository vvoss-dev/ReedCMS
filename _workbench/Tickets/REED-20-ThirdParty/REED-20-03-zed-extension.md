# REED-20-03: Zed Extension

**Status**: Open  
**Priority**: Low  
**Complexity**: Medium  
**Layer**: Third-Party Integration  
**Dependencies**: REED-20-01 (MCP Server Library)

## Overview

Develop a Zed editor extension that integrates ReedCMS using the native Zed extension API and MCP protocol support, providing lightweight content management optimised for Zed's performance-first philosophy.

## Objectives

1. **Native Zed Integration**: Leverage Zed's built-in MCP support
2. **Language Server Protocol**: Custom LSP for ReedCMS key completion
3. **Minimal UI**: Inline commands and quick actions (no heavy sidebars)
4. **Performance First**: Fast startup, minimal overhead
5. **Keyboard-Centric**: Vim-style command mode integration
6. **MCP Integration**: Direct use of REED-20-01 MCP server

## Architecture

### Extension Structure
```
reedcms-zed/
├── extension.toml           # Zed extension manifest
├── Cargo.toml              # Rust-based extension
├── src/
│   ├── lib.rs              # Extension entry point
│   ├── mcp_client.rs       # MCP communication
│   ├── lsp.rs              # Language Server Protocol
│   ├── commands.rs         # Zed command bindings
│   └── snippets.rs         # Code snippets
└── languages/
    └── reed-csv/
        ├── config.toml     # Language configuration
        └── highlights.scm  # Tree-sitter syntax
```

## Distribution

Published to Zed Extensions Marketplace.

## Related Tickets

- **REED-20-01**: MCP Server Library (dependency)
- **REED-20-02**: VS Code Extension
- **REED-20-04**: JetBrains Extension
