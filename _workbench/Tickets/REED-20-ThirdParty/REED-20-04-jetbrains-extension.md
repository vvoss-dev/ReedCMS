# REED-20-04: JetBrains Extension

**Status**: Open  
**Priority**: Low  
**Complexity**: High  
**Layer**: Third-Party Integration  
**Dependencies**: REED-20-01 (MCP Server Library)

## Overview

Develop a JetBrains IDE plugin (IntelliJ IDEA, WebStorm, PyCharm, etc.) that integrates ReedCMS using the IntelliJ Platform SDK and MCP protocol.

## Objectives

1. **Platform SDK Integration**: Use IntelliJ Platform for all JetBrains IDEs
2. **Tool Window**: Dedicated ReedCMS panel
3. **Advanced Refactoring**: Key renaming, layout migration
4. **Visual CSV Editor**: Table-based editing
5. **MCP Integration**: Bridge to reed-mcp-server
6. **Multi-IDE Support**: Single plugin for all JetBrains IDEs

## Distribution

Published to JetBrains Marketplace for all IDEs (IntelliJ IDEA, WebStorm, PyCharm, PhpStorm, etc.).

## Related Tickets

- **REED-20-01**: MCP Server Library (dependency)
- **REED-20-02**: VS Code Extension
- **REED-20-03**: Zed Extension
