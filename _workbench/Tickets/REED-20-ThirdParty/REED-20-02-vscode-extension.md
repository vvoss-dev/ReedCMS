# REED-20-02: VS Code Extension

**Status**: Open
**Priority**: Medium
**Complexity**: High
**Layer**: Third-Party Integration
**Dependencies**: REED-20-01 (MCP Server Library)

## Overview

Develop a Visual Studio Code extension that integrates ReedCMS directly into the VS Code environment, providing content management, layout creation, and AI-assisted content generation within the editor.

## Objectives

1. **Native VS Code Integration**: Sidebar panel for ReedCMS project management
2. **Content Editing**: Inline editing of text/route/meta entries with validation
3. **Layout Management**: Visual layout creation and hierarchy management
4. **MCP Integration**: Use REED-20-01 MCP server for backend communication
5. **AI-Assisted Content**: Claude integration for content generation via MCP
6. **Syntax Highlighting**: Custom highlighting for .csv files and Jinja templates
7. **IntelliSense**: Auto-completion for keys, languages, layouts
8. **Preview Mode**: Live preview of rendered layouts

## Architecture

### Package Structure
```
reedcms-vscode/
├── package.json              # VS Code extension manifest
├── src/
│   ├── extension.ts          # Extension entry point
│   ├── mcpClient.ts          # MCP client wrapper (uses reed-mcp-server)
│   ├── sidebar/
│   │   ├── projectView.ts    # Project tree view
│   │   ├── layoutView.ts     # Layout hierarchy view
│   │   └── contentView.ts    # Content browser
│   ├── editors/
│   │   ├── csvEditor.ts      # Custom .csv editor with table view
│   │   ├── textEditor.ts     # Text entry editor
│   │   └── layoutEditor.ts   # Visual layout editor
│   ├── commands/
│   │   ├── dataCommands.ts   # set:text, get:text, list:text
│   │   ├── layoutCommands.ts # init:layout, manage layouts
│   │   └── agentCommands.ts  # AI content generation
│   ├── providers/
│   │   ├── completionProvider.ts  # IntelliSense for keys/languages
│   │   ├── hoverProvider.ts       # Hover tooltips for keys
│   │   └── diagnosticProvider.ts  # Validation warnings
│   └── preview/
│       ├── previewPanel.ts   # WebView for layout preview
│       └── liveReload.ts     # Hot reload on file changes
├── syntaxes/
│   ├── reed-csv.tmLanguage.json    # CSV syntax highlighting
│   └── reed-jinja.tmLanguage.json  # Enhanced Jinja highlighting
└── resources/
    └── icons/                # Custom icons for ReedCMS files
```

### MCP Communication Pattern
```typescript
// src/mcpClient.ts
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

export class ReedMCPClient {
  private client: Client;

  async connect(projectPath: string): Promise<void> {
    // Start reed-mcp-server process
    const transport = new StdioClientTransport({
      command: 'reed-mcp-server',
      args: ['--project', projectPath]
    });

    this.client = new Client({
      name: 'reedcms-vscode',
      version: '1.0.0'
    }, {
      capabilities: {}
    });

    await this.client.connect(transport);
  }

  async setText(key: string, value: string, desc: string): Promise<void> {
    const result = await this.client.callTool({
      name: 'reed_set_text',
      arguments: { key, value, description: desc }
    });
    return result.content;
  }

  async getText(key: string): Promise<string> {
    const result = await this.client.callTool({
      name: 'reed_get_text',
      arguments: { key }
    });
    return result.content;
  }

  async generateContent(prompt: string, layout: string): Promise<string> {
    const result = await this.client.callTool({
      name: 'reed_agent_generate',
      arguments: {
        prompt,
        context: { layout },
        agent: 'claude'
      }
    });
    return result.content;
  }
}
```

## Features

### 1. Sidebar Panel
**ReedCMS Explorer** with three views:

- **Project View**:
  - `.reed/` directory browser
  - Quick access to text.csv, routes.csv, meta.csv
  - CSV statistics (entries count, languages)

- **Layout View**:
  - Hierarchical layout tree
  - Parent-child relationships
  - Quick create/delete/rename

- **Content View**:
  - Searchable key browser
  - Filter by layout/language
  - Quick edit inline

### 2. Custom CSV Editor
**Table View** for .reed/*.csv files:
```
┌─────────────────────────┬──────────────────┬────────────────────────┐
│ Key                     │ Value            │ Description            │
├─────────────────────────┼──────────────────┼────────────────────────┤
│ landing.hero.title@de   │ Entwickler       │ Hero headline          │
│ landing.hero.title@en   │ Developer        │ Hero headline          │
│ knowledge.page.title@de │ Wissen           │ Page title             │
└─────────────────────────┴──────────────────┴────────────────────────┘
```

- Inline editing with validation
- Add/delete rows
- Sort by column
- Search/filter
- Syntax highlighting
- Auto-save with backup

### 3. Command Palette Integration
```
> ReedCMS: Set Text Entry
> ReedCMS: Create New Layout
> ReedCMS: Generate Content with AI
> ReedCMS: Validate Project
> ReedCMS: Start Server
> ReedCMS: Preview Layout
> ReedCMS: Migrate Text from File
```

### 4. IntelliSense & Auto-Completion
**Context-aware suggestions**:

```jinja
{{ "landing.hero.↓ }}
         ↓
    ┌──────────────────────────┐
    │ landing.hero.title       │
    │ landing.hero.subtitle    │
    │ landing.hero.cta         │
    └──────────────────────────┘

{{ "landing.hero.title" | text("↓") }}
                                ↓
                        ┌──────────────┐
                        │ de           │
                        │ en           │
                        │ fr           │
                        └──────────────┘
```

### 5. Live Preview Panel
**WebView-based preview**:
- Renders layout using local ReedCMS server
- Hot reload on file changes
- Mouse/Touch/Reader variant switcher
- Language switcher
- Responsive viewport preview

### 6. AI Content Generation
**Integrated with Claude via MCP**:

```typescript
// User selects text in editor
// Right-click → "ReedCMS: Generate Translation"
// AI generates translation using context from layout

const translation = await reedClient.generateContent(
  `Translate "${selectedText}" from German to English for ${layoutName} layout`,
  layoutName
);
```

## Configuration

### VS Code Settings
```json
{
  "reedcms.projectPath": "${workspaceFolder}",
  "reedcms.serverPort": 8080,
  "reedcms.defaultLanguage": "de",
  "reedcms.mcpServer.path": "reed-mcp-server",
  "reedcms.mcpServer.autoStart": true,
  "reedcms.preview.autoReload": true,
  "reedcms.preview.defaultVariant": "mouse",
  "reedcms.ai.provider": "anthropic",
  "reedcms.ai.model": "claude-3-5-sonnet-20241022",
  "reedcms.csv.tableView": true,
  "reedcms.validation.onSave": true
}
```

### Extension Manifest (package.json)
```json
{
  "name": "reedcms",
  "displayName": "ReedCMS",
  "description": "Manage ReedCMS projects directly in VS Code",
  "version": "1.0.0",
  "publisher": "vvoss-dev",
  "engines": {
    "vscode": "^1.80.0"
  },
  "categories": ["Other", "Programming Languages"],
  "activationEvents": [
    "workspaceContains:.reed/text.csv"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "viewsContainers": {
      "activitybar": [{
        "id": "reedcms",
        "title": "ReedCMS",
        "icon": "resources/icons/reed.svg"
      }]
    },
    "views": {
      "reedcms": [
        { "id": "reedcms.project", "name": "Project" },
        { "id": "reedcms.layouts", "name": "Layouts" },
        { "id": "reedcms.content", "name": "Content" }
      ]
    },
    "commands": [
      {
        "command": "reedcms.setText",
        "title": "ReedCMS: Set Text Entry"
      },
      {
        "command": "reedcms.createLayout",
        "title": "ReedCMS: Create New Layout"
      },
      {
        "command": "reedcms.generateContent",
        "title": "ReedCMS: Generate Content with AI"
      }
    ],
    "languages": [
      {
        "id": "reed-csv",
        "extensions": [".csv"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "reed-csv",
        "scopeName": "source.reed.csv",
        "path": "./syntaxes/reed-csv.tmLanguage.json"
      }
    ]
  },
  "dependencies": {
    "@modelcontextprotocol/sdk": "^0.5.0"
  }
}
```

## Distribution

### VS Code Marketplace
```bash
# Build extension
npm run compile
vsce package

# Publish to marketplace
vsce publish
```

### Installation
```bash
# From marketplace
code --install-extension vvoss-dev.reedcms

# From VSIX file
code --install-extension reedcms-1.0.0.vsix
```

## Benefits

- ✅ **Native IDE Integration**: Manage ReedCMS without leaving VS Code
- ✅ **Visual Editing**: Table view for CSV files, inline editing
- ✅ **AI-Powered**: Content generation using Claude via MCP
- ✅ **Developer Experience**: IntelliSense, validation, auto-completion
- ✅ **Live Preview**: See changes immediately in preview panel
- ✅ **Productivity**: Command palette, keyboard shortcuts, quick actions
- ✅ **MCP Foundation**: Built on REED-20-01 MCP server library

## Acceptance Criteria

- [ ] Extension activates when `.reed/` directory detected
- [ ] Sidebar shows project/layout/content views
- [ ] Custom CSV editor with table view
- [ ] All CLI commands accessible via command palette
- [ ] IntelliSense for keys, languages, layouts
- [ ] Live preview panel with hot reload
- [ ] AI content generation via MCP
- [ ] Syntax highlighting for CSV and Jinja files
- [ ] Published to VS Code Marketplace
- [ ] Documentation and usage examples

## Related Tickets

- **REED-20-01**: MCP Server Library (dependency)
- **REED-20-03**: Zed Extension (similar architecture)
- **REED-20-04**: JetBrains Extension (similar features)
- **REED-04-10**: CLI Agent Commands (MCP integration foundation)
