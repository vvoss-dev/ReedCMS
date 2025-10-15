# REED-19-04: Encoded Log System

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid) - minimal code and professional inline documentation
- **File Naming**: File name = Unique theme - crystal clear what single topic this file handles
- **Files**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Shared Functions**: Same patterns = One shared function
- **Testing**: Separate test files as `{name}.test.rs` - never inline `#[cfg(test)]` modules
- **Avoid**: Avoid Swiss Army knife functions
- **Avoid**: Generic file names like `handler.rs`, `utils.rs`
- **Templates**: See `_workbench/Tickets/templates/service-template.md` for complete implementation guide
- **Testing**: See `_workbench/Tickets/templates/service-template.test.md` for test structure

## Ticket Information
- **ID**: REED-19-04
- **Title**: Encoded Log System
- **Layer**: ReedBase Layer (REED-19)
- **Priority**: High
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-19-01 (Registry & Dictionary), REED-19-03 (Binary Delta Versioning)
- **Estimated Time**: 4 days

## Objective

Implement encoded version log system using integer codes from registries instead of strings. Achieve 50% smaller log files and 5x faster parsing compared to plain text logs.

## Requirements

### Log Format

**Plain text (old way):**
```
1736860900|update|admin|1736860800|2500|15|sha256:abc123
```

**Encoded with CRC32 validation (new way):**
```
REED|00000052|1736860900|2|1|1736860800|2500|15|sha256:abc123|uuid002|A1B2C3D4
```

**Format specification:**
```
{magic}|{length}|{timestamp}|{action_code}|{user_code}|{base_version}|{size}|{rows}|{hash}|{frame_id}|{crc32}
```

**Fields:**
- `magic`: "REED" (4 bytes) - Corruption detection at startup
- `length`: Entry size in bytes (8 bytes hex) - Validates complete write
- `timestamp`: Unix timestamp (u64)
- `action_code`: Action code from dictionary (u8)
- `user_code`: User code from dictionary (u32)
- `base_version`: Previous version timestamp (u64)
- `size`: Delta size in bytes (usize)
- `rows`: Number of rows affected (usize)
- `hash`: SHA-256 hash of delta
- `frame_id`: Frame UUID if operation is part of coordinated batch, "n/a" otherwise
- `crc32`: CRC32 checksum of data portion (8 bytes hex)

**Frame Integration:**

When operations are part of a Frame (coordinated batch), the `frame_id` field links all related operations:

```
# Schema migration frame (uuid002)
REED|00000065|1736860800|2|1|1736860700|1500|10|sha256:abc|uuid002|A1B2C3D4  # text table
REED|00000065|1736860800|2|1|1736860700|800|5|sha256:def|uuid002|E5F6G7H8   # routes table
REED|00000065|1736860800|2|1|1736860750|1200|8|sha256:ghi|uuid002|I9J0K1L2  # meta table

# Regular operation (no frame)
REED|00000058|1736860900|2|1|1736860800|500|2|sha256:jkl|n/a|M3N4O5P6
```

This enables:
- Fast lookup of all operations in a frame
- Point-in-time recovery using frame snapshots
- Rollback of coordinated operations
- Audit trail for batch changes

### Dictionary Mappings

**actions.dict:**
```csv
code|name|description
0|delete|Delete operation
1|create|Create new entry
2|update|Update existing entry
3|rollback|Rollback to previous version
4|compact|Compact/cleanup old versions
5|init|Initialise table
```

**users.dict:**
```csv
code|username|created_at
0|system|1736850000
1|admin|1736860000
```

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Write log entry | < 5ms | Encode + append to file |
| Read log entry | < 1ms | Parse single line |
| Read full log (1000 entries) | < 50ms | Parse all entries |
| Decode action code | < 100ns | O(1) HashMap lookup |
| Decode user code | < 100ns | O(1) HashMap lookup |
| Log file size | 50% of plain | Integer codes vs strings |

## Implementation Files

### Primary Implementation

**`reedbase/src/log/encoder.rs`**

One file = Log encoding only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log encoding for version history.
//!
//! Encodes log entries using integer codes from registries.

use crate::types::{ReedResult, ReedError, LogEntry};
use crate::registry;

/// Encode log entry to string.
///
/// ## Input
/// - `entry`: Log entry to encode
///
/// ## Output
/// - `ReedResult<String>`: Encoded log line
///
/// ## Performance
/// - < 100μs typical (2 dictionary lookups + string formatting)
///
/// ## Error Conditions
/// - UnknownAction: Action name not found in actions.dict
/// - UnknownUser: User not found in users.dict
///
/// ## Example Usage
/// ```rust
/// let entry = LogEntry {
///     timestamp: 1736860900,
///     action: "update".to_string(),
///     user: "admin".to_string(),
///     base_version: 1736860800,
///     size: 2500,
///     rows: 15,
///     hash: "sha256:abc123".to_string(),
///     frame_id: Some(Uuid::parse_str("uuid002")?),
/// };
/// let encoded = encode_log_entry(&entry)?;
/// // "1736860900|2|1|1736860800|2500|15|sha256:abc123|uuid002"
/// ```
pub fn encode_log_entry(entry: &LogEntry) -> ReedResult<String> {
    let action_code = registry::get_action_code(&entry.action)?;
    let user_code = registry::get_or_create_user_code(&entry.user)?;
    
    let frame_id_str = entry.frame_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "n/a".to_string());
    
    Ok(format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        entry.timestamp,
        action_code,
        user_code,
        entry.base_version,
        entry.size,
        entry.rows,
        entry.hash,
        frame_id_str
    ))
}

/// Encode multiple log entries.
///
/// ## Input
/// - `entries`: Log entries to encode
///
/// ## Output
/// - `ReedResult<String>`: Encoded log (newline-separated)
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 100μs per entry
///
/// ## Error Conditions
/// - UnknownAction: Action name not found
/// - UnknownUser: User not found
///
/// ## Example Usage
/// ```rust
/// let entries = vec![entry1, entry2, entry3];
/// let encoded = encode_log_entries(&entries)?;
/// fs::write("version.log", encoded)?;
/// ```
pub fn encode_log_entries(entries: &[LogEntry]) -> ReedResult<String> {
    let mut lines = Vec::new();
    
    for entry in entries {
        lines.push(encode_log_entry(entry)?);
    }
    
    Ok(lines.join("\n"))
}

/// Calculate encoded size vs plain text size.
///
/// ## Input
/// - `entries`: Log entries
///
/// ## Output
/// - `(usize, usize)`: (encoded_size, plain_size) in bytes
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 1ms for 100 entries
///
/// ## Example Usage
/// ```rust
/// let (encoded, plain) = calculate_size_savings(&entries);
/// let savings = ((plain - encoded) as f64 / plain as f64) * 100.0;
/// println!("Savings: {:.1}%", savings);
/// ```
pub fn calculate_size_savings(entries: &[LogEntry]) -> ReedResult<(usize, usize)> {
    let encoded = encode_log_entries(entries)?.len();
    
    let mut plain_size = 0;
    for entry in entries {
        plain_size += format!(
            "{}|{}|{}|{}|{}|{}|{}",
            entry.timestamp,
            entry.action,
            entry.user,
            entry.base_version,
            entry.size,
            entry.rows,
            entry.hash
        ).len() + 1; // +1 for newline
    }
    
    Ok((encoded, plain_size))
}
```

**`reedbase/src/log/decoder.rs`**

One file = Log decoding only. NO other responsibilities.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

//! Log decoding for version history.
//!
//! Decodes log entries from integer codes to human-readable format.

use crate::types::{ReedResult, ReedError, LogEntry};
use crate::registry;

/// Decode log entry from string.
///
/// ## Input
/// - `line`: Encoded log line
///
/// ## Output
/// - `ReedResult<LogEntry>`: Decoded log entry
///
/// ## Performance
/// - < 1ms typical (2 dictionary lookups + parsing)
///
/// ## Error Conditions
/// - ParseError: Invalid format or field count
/// - UnknownActionCode: Action code not found
/// - UnknownUserCode: User code not found
///
/// ## Example Usage
/// ```rust
/// let line = "1736860900|2|1|1736860800|2500|15|sha256:abc123|uuid002";
/// let entry = decode_log_entry(line)?;
/// assert_eq!(entry.action, "update");
/// assert_eq!(entry.user, "admin");
/// assert!(entry.frame_id.is_some());
/// ```
pub fn decode_log_entry(line: &str) -> ReedResult<LogEntry> {
    let parts: Vec<&str> = line.split('|').collect();
    
    // Support both old (7 fields) and new (8 fields with frame_id) formats
    if parts.len() != 7 && parts.len() != 8 {
        return Err(ReedError::ParseError {
            reason: format!("Expected 7 or 8 fields, got {}", parts.len()),
        });
    }
    
    let timestamp = parts[0].parse::<u64>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid timestamp: {}", e),
        })?;
    
    let action_code = parts[1].parse::<u8>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid action code: {}", e),
        })?;
    
    let user_code = parts[2].parse::<u32>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid user code: {}", e),
        })?;
    
    let base_version = parts[3].parse::<u64>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid base version: {}", e),
        })?;
    
    let size = parts[4].parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid size: {}", e),
        })?;
    
    let rows = parts[5].parse::<usize>()
        .map_err(|e| ReedError::ParseError {
            reason: format!("Invalid rows: {}", e),
        })?;
    
    let hash = parts[6].to_string();
    
    // Parse frame_id if present (new format)
    let frame_id = if parts.len() == 8 {
        match parts[7] {
            "n/a" | "" => None,
            uuid_str => Some(Uuid::parse_str(uuid_str)
                .map_err(|e| ReedError::ParseError {
                    reason: format!("Invalid frame_id UUID: {}", e),
                })?),
        }
    } else {
        None  // Old format without frame_id
    };
    
    // Decode codes to names
    let action = registry::get_action_name(action_code)?;
    let user = registry::get_username(user_code)?;
    
    Ok(LogEntry {
        timestamp,
        action,
        user,
        base_version,
        size,
        rows,
        hash,
        frame_id,
    })
}

/// Decode multiple log entries.
///
/// ## Input
/// - `content`: Encoded log content (newline-separated)
///
/// ## Output
/// - `ReedResult<Vec<LogEntry>>`: Decoded log entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 1ms per entry
/// - < 50ms for 1000 entries
///
/// ## Error Conditions
/// - ParseError: Invalid format
/// - UnknownActionCode: Action code not found
/// - UnknownUserCode: User code not found
///
/// ## Example Usage
/// ```rust
/// let content = fs::read_to_string("version.log")?;
/// let entries = decode_log_entries(&content)?;
/// for entry in entries {
///     println!("{} - {} by {}", entry.timestamp, entry.action, entry.user);
/// }
/// ```
pub fn decode_log_entries(content: &str) -> ReedResult<Vec<LogEntry>> {
    let mut entries = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        
        entries.push(decode_log_entry(line).map_err(|e| {
            ReedError::ParseError {
                reason: format!("Line {}: {}", line_num + 1, e),
            }
        })?);
    }
    
    Ok(entries)
}

/// Filter log entries by action.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `action`: Action name to filter by
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```rust
/// let updates = filter_by_action(&entries, "update");
/// println!("Found {} update operations", updates.len());
/// ```
pub fn filter_by_action<'a>(entries: &'a [LogEntry], action: &str) -> Vec<&'a LogEntry> {
    entries.iter()
        .filter(|e| e.action == action)
        .collect()
}

/// Filter log entries by user.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `user`: Username to filter by
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```rust
/// let admin_actions = filter_by_user(&entries, "admin");
/// println!("Admin performed {} actions", admin_actions.len());
/// ```
pub fn filter_by_user<'a>(entries: &'a [LogEntry], user: &str) -> Vec<&'a LogEntry> {
    entries.iter()
        .filter(|e| e.user == user)
        .collect()
}

/// Filter log entries by time range.
///
/// ## Input
/// - `entries`: Log entries to filter
/// - `start`: Start timestamp (inclusive)
/// - `end`: End timestamp (inclusive)
///
/// ## Output
/// - `Vec<&LogEntry>`: Filtered entries
///
/// ## Performance
/// - O(n) where n = number of entries
/// - < 10ms for 1000 entries
///
/// ## Example Usage
/// ```rust
/// let yesterday = now() - 86400;
/// let recent = filter_by_time_range(&entries, yesterday, now());
/// println!("Last 24h: {} operations", recent.len());
/// ```
pub fn filter_by_time_range<'a>(
    entries: &'a [LogEntry],
    start: u64,
    end: u64,
) -> Vec<&'a LogEntry> {
    entries.iter()
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .collect()
}
```

**`reedbase/src/types.rs`** (additions)

```rust
/// Log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub action: String,
    pub user: String,
    pub base_version: u64,
    pub size: usize,
    pub rows: usize,
    pub hash: String,
    pub frame_id: Option<Uuid>,  // Frame UUID if part of coordinated batch
}

/// Additional ReedBase errors.
#[derive(Error, Debug)]
pub enum ReedError {
    // ... (existing errors)
    
    #[error("Parse error: {reason}")]
    ParseError {
        reason: String,
    },
    
    #[error("Unknown action code: {code}")]
    UnknownActionCode {
        code: u8,
    },
    
    #[error("Unknown user code: {code}")]
    UnknownUserCode {
        code: u32,
    },
}
```

### Test Files

**`reedbase/src/log/encoder.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_entry() -> LogEntry {
        LogEntry {
            timestamp: 1736860900,
            action: "update".to_string(),
            user: "admin".to_string(),
            base_version: 1736860800,
            size: 2500,
            rows: 15,
            hash: "sha256:abc123".to_string(),
        }
    }
    
    #[test]
    fn test_encode_log_entry() {
        let entry = create_test_entry();
        let encoded = encode_log_entry(&entry).unwrap();
        
        let parts: Vec<&str> = encoded.split('|').collect();
        assert_eq!(parts.len(), 7);
        assert_eq!(parts[0], "1736860900");
        assert_eq!(parts[3], "1736860800");
        assert_eq!(parts[4], "2500");
        assert_eq!(parts[5], "15");
        assert_eq!(parts[6], "sha256:abc123");
    }
    
    #[test]
    fn test_encode_multiple_entries() {
        let entry1 = create_test_entry();
        let mut entry2 = create_test_entry();
        entry2.timestamp = 1736861000;
        
        let encoded = encode_log_entries(&[entry1, entry2]).unwrap();
        let lines: Vec<&str> = encoded.lines().collect();
        
        assert_eq!(lines.len(), 2);
    }
    
    #[test]
    fn test_calculate_size_savings() {
        let entries = vec![create_test_entry(); 10];
        let (encoded_size, plain_size) = calculate_size_savings(&entries).unwrap();
        
        assert!(encoded_size < plain_size);
        let savings_percent = ((plain_size - encoded_size) as f64 / plain_size as f64) * 100.0;
        assert!(savings_percent > 30.0); // At least 30% savings
    }
}
```

**`reedbase/src/log/decoder.test.rs`**

Separate test file. Each test = one specific behaviour.

```rust
// Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_log_entry() {
        // Assuming action code 2 = "update", user code 1 = "admin"
        let line = "1736860900|2|1|1736860800|2500|15|sha256:abc123";
        let entry = decode_log_entry(line).unwrap();
        
        assert_eq!(entry.timestamp, 1736860900);
        assert_eq!(entry.action, "update");
        assert_eq!(entry.user, "admin");
        assert_eq!(entry.base_version, 1736860800);
        assert_eq!(entry.size, 2500);
        assert_eq!(entry.rows, 15);
        assert_eq!(entry.hash, "sha256:abc123");
    }
    
    #[test]
    fn test_decode_invalid_field_count() {
        let line = "1736860900|2|1|1736860800"; // Only 4 fields
        let result = decode_log_entry(line);
        
        assert!(matches!(result, Err(ReedError::ParseError { .. })));
    }
    
    #[test]
    fn test_decode_invalid_timestamp() {
        let line = "invalid|2|1|1736860800|2500|15|sha256:abc123";
        let result = decode_log_entry(line);
        
        assert!(matches!(result, Err(ReedError::ParseError { .. })));
    }
    
    #[test]
    fn test_decode_multiple_entries() {
        let content = "1736860900|2|1|1736860800|2500|15|sha256:abc123\n1736861000|2|1|1736860900|2600|16|sha256:def456";
        let entries = decode_log_entries(content).unwrap();
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].timestamp, 1736860900);
        assert_eq!(entries[1].timestamp, 1736861000);
    }
    
    #[test]
    fn test_decode_empty_lines() {
        let content = "1736860900|2|1|1736860800|2500|15|sha256:abc123\n\n1736861000|2|1|1736860900|2600|16|sha256:def456\n";
        let entries = decode_log_entries(content).unwrap();
        
        assert_eq!(entries.len(), 2);
    }
    
    #[test]
    fn test_filter_by_action() {
        let entry1 = LogEntry {
            timestamp: 1736860900,
            action: "update".to_string(),
            user: "admin".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entry2 = LogEntry {
            timestamp: 1736861000,
            action: "delete".to_string(),
            user: "admin".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entries = vec![entry1, entry2];
        let updates = filter_by_action(&entries, "update");
        
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].action, "update");
    }
    
    #[test]
    fn test_filter_by_user() {
        let entry1 = LogEntry {
            timestamp: 1736860900,
            action: "update".to_string(),
            user: "admin".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entry2 = LogEntry {
            timestamp: 1736861000,
            action: "update".to_string(),
            user: "editor".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entries = vec![entry1, entry2];
        let admin_actions = filter_by_user(&entries, "admin");
        
        assert_eq!(admin_actions.len(), 1);
        assert_eq!(admin_actions[0].user, "admin");
    }
    
    #[test]
    fn test_filter_by_time_range() {
        let entry1 = LogEntry {
            timestamp: 1736860900,
            action: "update".to_string(),
            user: "admin".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entry2 = LogEntry {
            timestamp: 1736861000,
            action: "update".to_string(),
            user: "admin".to_string(),
            base_version: 0,
            size: 0,
            rows: 0,
            hash: "".to_string(),
        };
        
        let entries = vec![entry1, entry2];
        let filtered = filter_by_time_range(&entries, 1736860950, 1736861100);
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].timestamp, 1736861000);
    }
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Encode single entry | < 100μs |
| Decode single entry | < 1ms |
| Encode 100 entries | < 10ms |
| Decode 1000 entries | < 50ms |
| Dictionary lookup | < 100ns |
| Filter 1000 entries | < 10ms |
| Log file size | 50% of plain text |

## Error Conditions

- **ParseError**: Invalid log format or field count
- **UnknownActionCode**: Action code not in actions.dict
- **UnknownUserCode**: User code not in users.dict
- **UnknownAction**: Action name not found (encoding)
- **UnknownUser**: User not found (encoding - creates new user)

## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| log_validation_latency | Histogram | ms | <1 | >5 | log.rs:validate_log() |
| log_decode_latency | Histogram | ms | <1 | >5 | log.rs:decode_entry() |
| log_encode_latency | Histogram | μs | <100 | >500 | log.rs:encode_entry() |
| crc_check_time | Histogram | μs | <50 | >200 | log.rs:validate_crc() |
| corrupted_entries | Counter | count | 0 | >0 | log.rs:validate_log() |
| log_parse_errors | Counter | count | <0.1% | >1% | log.rs:decode_entry() |

### Alert Rules

**CRITICAL Alerts:**
- `corrupted_entries > 0` for 1 minute → "Log corruption detected - data integrity issue"
- `log_parse_errors > 1%` for 5 minutes → "High log parse error rate - investigate format issues"

**WARNING Alerts:**
- `log_validation_latency p99 > 5ms` for 5 minutes → "Log validation slow - check CRC performance"
- `crc_check_time p99 > 200μs` for 10 minutes → "CRC validation slower than expected"

### Implementation

```rust
use crate::reedbase::metrics::global as metrics;
use std::time::Instant;

pub fn validate_log(log_path: &Path) -> ReedResult<ValidationReport> {
    let start = Instant::now();
    let report = validate_log_inner(log_path)?;
    
    metrics().record(Metric {
        name: "log_validation_latency".to_string(),
        value: start.elapsed().as_millis() as f64,
        unit: MetricUnit::Milliseconds,
        tags: hashmap!{ "log" => log_path.to_string_lossy().to_string() },
    });
    
    metrics().record(Metric {
        name: "corrupted_entries".to_string(),
        value: report.corrupted_count as f64,
        unit: MetricUnit::Count,
        tags: hashmap!{ "log" => log_path.to_string_lossy().to_string() },
    });
    
    Ok(report)
}

pub fn validate_crc(entry: &str, expected_crc: u32) -> ReedResult<bool> {
    let start = Instant::now();
    let is_valid = validate_crc_inner(entry, expected_crc)?;
    
    metrics().record(Metric {
        name: "crc_check_time".to_string(),
        value: start.elapsed().as_nanos() as f64 / 1000.0, // Convert to μs
        unit: MetricUnit::Microseconds,
        tags: hashmap!{ "valid" => is_valid.to_string() },
    });
    
    Ok(is_valid)
}
```

### Collection Strategy

- **Sampling**: All operations
- **Aggregation**: 1-minute rolling window
- **Storage**: `.reedbase/metrics/log.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**log_validation_latency**: System startup performance
- Validation runs on every table load
- Slow validation delays system availability
- Indicates log file size or I/O issues

**crc_check_time**: Crash recovery efficiency
- CRC checked for EVERY log entry during recovery
- Sub-microsecond checks critical for fast recovery
- Degradation indicates CPU or memory pressure

**corrupted_entries**: Data integrity monitoring
- Should ALWAYS be zero in healthy system
- Any corruption indicates serious problem (power loss, disk failure, bug)
- Triggers immediate investigation and recovery

**log_parse_errors**: Log format health
- Low error rate expected (dictionary mismatches possible)
- High rates indicate bugs in encoding/decoding logic
- Affects system reliability and recoverability

## CLI Commands

```bash
# View log (decoded)
reed log:show users
# Output:
# 1736860900 - update by admin (15 rows, 2500 bytes)
# 1736860800 - create by system (10 rows, 1500 bytes)

# Filter by action
reed log:show users --action update
# Output: Only update operations

# Filter by user
reed log:show users --user admin
# Output: Only admin operations

# Filter by time range
reed log:show users --since 2025-01-14 --until 2025-01-15
# Output: Operations in date range

# Show raw encoded log
reed log:show users --raw
# Output: Raw encoded lines (for debugging)
```

## Acceptance Criteria

- [ ] Encode log entry using action/user codes with CRC32
- [ ] Add magic bytes ("REED") and length field to entries
- [ ] Calculate and append CRC32 checksum
- [ ] Decode log entry to human-readable format
- [ ] Validate magic bytes on decode
- [ ] Validate CRC32 checksum on decode
- [ ] Encode multiple entries to file
- [ ] Decode multiple entries from file
- [ ] `validate_and_truncate_log()` removes corrupted entries
- [ ] Auto-truncation on startup for crash recovery
- [ ] Calculate size savings (50%+ vs plain text)
- [ ] Filter log entries by action
- [ ] Filter log entries by user
- [ ] Filter log entries by time range
- [ ] Handle empty lines gracefully
- [ ] Return specific errors for invalid formats (ParseError, CorruptedLogEntry)
- [ ] All tests pass with 100% coverage
- [ ] Performance targets met (< 150μs encode with CRC32)
- [ ] All code in BBC English
- [ ] All functions have complete documentation (Input/Output/Performance/Error Conditions/Example Usage)
- [ ] No Swiss Army knife functions (each function = one job)
- [ ] Separate test files as `encoder.test.rs` and `decoder.test.rs`

## Dependencies

**Requires**: 
- REED-19-01 (Registry & Dictionary - for code lookups)
- REED-19-03 (Binary Delta Versioning - generates log entries)

**Blocks**: None

## References
- Service Template: `_workbench/Tickets/templates/service-template.md`
- Service Test Template: `_workbench/Tickets/templates/service-template.test.md`
- REED-19-00: Layer Overview

## Notes

**Encoding Benefits:**

| Metric | Plain Text | Encoded | Improvement |
|--------|-----------|---------|-------------|
| Log size (1000 entries) | ~150KB | ~75KB | 50% smaller |
| Parse time (1000 entries) | ~250ms | ~50ms | 5x faster |
| Action field size | 6-8 chars | 1 char | 6-8x smaller |
| User field size | 5-20 chars | 1-4 chars | 5-20x smaller |

**Why Integer Codes?**
- **Smaller files**: Single digits vs multi-character strings
- **Faster parsing**: Integer parsing faster than string comparison
- **Better compression**: XZ compresses repeating integers better
- **Type safety**: Invalid codes detected at dictionary level

**Trade-offs:**
- **Pro**: 50% smaller log files (critical for long-running tables with 10k+ versions)
- **Pro**: 5x faster log parsing (critical for version history queries)
- **Pro**: Better XZ compression (integers compress better than strings)
- **Con**: Requires dictionary lookups (mitigated by HashMap caching - 100ns per lookup)
- **Con**: Dictionary must be maintained (automated via get_or_create_user_code)

**Crash Recovery Integration:**

This ticket provides the foundation for crash recovery via:
1. **Magic bytes ("REED")**: Instant corruption detection at startup
2. **CRC32 validation**: Detect partial writes and bit-rot
3. **Auto-truncation**: `validate_and_truncate_log()` removes corrupted entries on startup
4. **Length field**: Validates complete entry was written

**Recovery workflow** (integrated with REED-19-03):
```
Startup → Validate .log (truncate corruption) → Check .csv integrity → 
If .csv corrupt: Emergency recovery from last valid delta (see REED-19-03)
```

**Safety guarantees:**
- ✅ Partial writes detected and removed automatically
- ✅ Bit-rot detected via CRC32 mismatch
- ✅ Log always consistent after startup validation
- ✅ Maximum loss: Last incomplete entry only

**Future Enhancements:**
- Compressed log format (gzip per entry for huge logs)
- Incremental log parsing (stream processing)
- Log rotation (archive old entries)
- Log indexing (B-tree index for fast queries)
