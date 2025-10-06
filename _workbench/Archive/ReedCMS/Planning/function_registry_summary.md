# ReedCMS Function Registry Summary

**Generated**: 2025-01-04
**Total Functions**: 984 (excluding header)

## Statistics

### By Status
- **Active functions**: 401
- **Test functions** (`cfg_test`): 583

### By Module (Top 20)

```bash
# Count by directory
grep -v "^file_path" _workbench/functions_registry.csv | cut -d'|' -f1 | cut -d'/' -f2-3 | sort | uniq -c | sort -rn | head -20
```

Results:
```
 193 reedcms/taxonomy
  95 reedcms/cli
  90 reedcms/security
  72 reedcms/reedbase
  48 reedcms/api
  43 reedcms/routing
  40 reedcms/backup
  36 reedcms/matrix
  36 reedcms/csv
  24 reedcms/templates
  21 reedcms/server
  14 reedcms/filters
  13 reedcms/auth
   3 reedcms/reed
   1 reedcms
```

## File Structure

The function registry is stored in **pipe-delimited CSV format** at:
`_workbench/functions_registry.csv`

### CSV Schema

| Column | Description | Example |
|--------|-------------|---------|
| `file_path` | Relative path from project root | `src/reedcms/cli/data_commands.rs` |
| `line_number` | Line where function starts | `243` |
| `function_name` | Function identifier | `get_text` |
| `signature` | Complete function signature | `pub fn get_text(args: &[String]) -> ReedResult<ReedResponse<String>>` |
| `description` | First line of doc comment | `Gets text content via CLI.` |
| `inputs` | Semicolon-separated parameters | `args: &[String]` |
| `return_type` | Return type | `ReedResult<ReedResponse<String>>` |
| `status` | `active` or `cfg_test` | `active` |

## Usage

### Find all functions in a module
```bash
grep "reedcms/cli/" _workbench/functions_registry.csv
```

### Find functions by name pattern
```bash
grep "|create_" _workbench/functions_registry.csv
```

### List all public API functions
```bash
grep "pub fn" _workbench/functions_registry.csv | grep -v "cfg_test"
```

### Count functions per file
```bash
cut -d'|' -f1 _workbench/functions_registry.csv | sort | uniq -c | sort -rn
```

## Excluded Items

- Impl blocks without explicit `fn` keyword
- Macro definitions
- Trait definitions (only their methods are included)
- Commented-out code

## Notes

- Functions in test modules (`*_test.rs`) are marked as `cfg_test`
- Functions within `#[cfg(test)]` blocks are marked as `cfg_test`
- Doc comments are extracted from the line immediately above the function
- Pipe characters (`|`) in content are escaped to `¦` to preserve CSV integrity
