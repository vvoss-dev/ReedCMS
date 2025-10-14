# REED-19: ReedBase Layer

## Layer Overview

**Status**: Design Phase  
**Priority**: Critical - Foundation Layer  
**Complexity**: Very High  
**Breaking Changes**: YES - Complete ReedBase rewrite

## Summary

Complete reimplementation of ReedBase as a versioned, concurrent-write capable database system with binary delta compression, row-level conflict resolution, and function memoization.

## Motivation

Current ReedBase (REED-02) has limitations:
- ❌ No versioning (only XZ backups)
- ❌ No concurrent writes (last-write-wins)
- ❌ No conflict resolution
- ❌ Inefficient full-file backups
- ❌ No row-level operations
- ❌ No function caching

**New ReedBase** provides:
- ✅ Git-like versioning with bsdiff deltas
- ✅ Concurrent writes with auto-merge
- ✅ Row-level conflict resolution
- ✅ Binary delta compression (95%+ savings)
- ✅ Function result caching
- ✅ Encoded metadata (efficient logs)

## Architecture

```
.reed/
├── registry/              # Global lookups
│   ├── actions.dict       # Action codes
│   └── users.dict         # User codes
│
├── tables/                # All data tables
│   ├── text/
│   │   ├── current.csv    # Current version
│   │   ├── {ts}.bsdiff    # Binary deltas
│   │   └── version.log    # Encoded metadata
│   │
│   ├── routes/
│   ├── meta/
│   ├── users/
│   └── ...
│
├── schema/                # Type definitions
│   └── *.schema.toml
│
├── cache/
│   ├── tables/
│   │   └── *.hashmap      # Table caches
│   └── functions/
│       └── *.cache        # Function result caches
│
└── config.toml
```

## Key Concepts

### 1. Universal Table Structure

Every table follows same pattern:
- `current.csv` - Active version
- `{timestamp}.bsdiff` - Binary deltas (XZ compressed)
- `version.log` - Encoded metadata (pipe-delimited integers)

### 2. Concurrent Writes

Multiple users can write simultaneously:
- Each creates own bsdiff delta
- System auto-merges at row level
- Conflicts detected and presented for resolution
- 90%+ of concurrent writes auto-merge (different rows)

### 3. Binary Delta Compression

- bsdiff creates small deltas (50-500 bytes typical)
- XZ compression reduces further
- Init = delta from empty file
- Rollback = apply delta chain (Git-like)
- 95%+ disk savings vs full snapshots

### 4. Encoded Metadata

version.log format: `timestamp|action|user|base|size|rows|hash`
- All integers (except hash)
- Lookup tables resolve to human-readable
- 5x faster parsing, 50% smaller files

### 5. Function Caching

Functions can cache results:
- Input hash → cached result
- TTL-based expiration
- LRU eviction
- 100-500x speedup for expensive operations

## Layer Tickets

- **REED-19-01**: Registry & Dictionary System
- **REED-19-02**: Universal Table API
- **REED-19-03**: Binary Delta Versioning (bsdiff)
- **REED-19-06**: Concurrent Write System
- **REED-19-06**: Row-Level CSV Merge
- **REED-19-07**: Conflict Resolution
- **REED-19-04**: Encoded Log System
- **REED-19-08**: Schema Validation
- **REED-19-09**: Function System & Caching
- **REED-19-10**: Smart Indices
- **REED-19-11**: CLI SQL-Like Query Interface (ReedQL)
- **REED-19-12**: Migration from REED-02
- **REED-19-13**: Performance Testing & Benchmarks

- **REED-19-14**: Documentation
## Impact Analysis

### What Changes

**Core Architecture:**
- `.reed/*.csv` → `.reed/tables/{name}/current.csv`
- Backup system → Versioning system (bsdiff deltas)
- Single-writer → Concurrent writers
- File-level ops → Row-level ops

**APIs:**
- `reedbase::get::text()` - Same API, different implementation
- `reedbase::set::text()` - Now returns conflict info
- New: `reedbase::merge::resolve_conflict()`
- New: `reedbase::version::list()`, `rollback()`

**CLI:**
- Existing commands mostly work (API compatible)
- New: `reed version:*` commands
- New: `reed conflict:*` commands
- New: `reed dict:*` commands
- Changed: `reed backup:*` → `reed version:*`

### What Stays Same

- ✅ Pipe-delimited CSV format
- ✅ Key nomenclature (`lowercase.with.dots@lang`)
- ✅ HashMap caching (O(1) lookups)
- ✅ XZ compression
- ✅ Environment fallback (`@dev`, `@prod`)
- ✅ Multi-language support

### What Gets Better

- ✅ **10x faster writes** (deltas vs full files)
- ✅ **95% less disk usage** (binary deltas)
- ✅ **Concurrent writes** (no more "database locked")
- ✅ **Complete history** (every change tracked)
- ✅ **Conflict resolution** (row-level merge)
- ✅ **Function caching** (100-500x speedup)

## Migration Strategy

### Phase 1: Parallel Implementation

Build new ReedBase alongside old:
- `src/reedcms/reedbase/` - Old (keep working)
- `src/reedcms/reedbase_v2/` - New implementation
- Feature flag: `reedbase_v2`

### Phase 2: Testing

Extensive testing:
- Unit tests (100% coverage target)
- Integration tests (real-world scenarios)
- Performance benchmarks
- Concurrent write stress tests
- Migration testing (old → new)

### Phase 3: Migration Command

```bash
reed migrate:reedbase-v2
```

Migrates existing `.reed/` to new structure:
- Moves CSVs to tables/
- Creates initial version.log entries
- Generates schemas
- Preserves all data

### Phase 4: Cutover

- Mark REED-02 as deprecated
- Make reedbase_v2 default
- Remove old code (after deprecation period)

## Risks

### High Risk

1. **Data Loss During Migration**
   - Mitigation: Automatic backup before migration
   - Mitigation: Dry-run mode with validation
   - Mitigation: Rollback capability

2. **Performance Regression**
   - Mitigation: Extensive benchmarks
   - Mitigation: Keep old implementation as fallback
   - Mitigation: Performance tests in CI

3. **Breaking Existing Integrations**
   - Mitigation: API compatibility layer
   - Mitigation: Gradual deprecation warnings
   - Mitigation: Comprehensive docs

### Medium Risk

1. **Concurrent Write Bugs**
   - Mitigation: Extensive stress testing
   - Mitigation: Conservative auto-merge (ask on doubt)

2. **Merge Logic Errors**
   - Mitigation: Formal test suite with all cases
   - Mitigation: Manual override always available

3. **Disk Space Increase (Many Versions)**
   - Mitigation: Configurable max_versions
   - Mitigation: Automatic cleanup
   - Mitigation: Deltas are tiny (95% savings)

## Success Criteria

- [ ] All REED-19 tickets completed
- [ ] 100% test coverage for core functionality
- [ ] Performance benchmarks show improvement
- [ ] Migration tested on real data
- [ ] Documentation complete
- [ ] Zero data loss in migration
- [ ] API compatibility maintained
- [ ] Concurrent writes work reliably

## Timeline Estimate

**Conservative: 3-4 weeks full-time**

- Week 1: Core tables + versioning (REED-19-01 to REED-19-03)
- Week 2: Concurrent writes + merge (REED-19-06 to REED-19-07)
- Week 3: Schemas + functions + indices + logs (REED-19-04 to REED-19-10)
- Week 4: ReedQL + migration + testing + docs (REED-19-11 to REED-19-14)

**Aggressive: 2 weeks** (if parallel development)

## Open Questions

1. **Function caching TTL defaults?**
   - Proposal: Configurable per function, default 1 hour

2. **Max versions per table?**
   - Proposal: Default 32 (like current backup), configurable

3. **Conflict resolution timeout?**
   - Proposal: Block write until resolved, or queue for async resolution

4. **Schema enforcement level?**
   - Proposal: Warning mode first, strict mode optional

5. **Keep REED-02 as fallback?**
   - Proposal: Yes, feature flag for 6 months, then remove

## References

- Current: REED-02 (Data Layer)
- Inspiration: Git (delta compression, versioning)
- Inspiration: PostgreSQL MVCC (concurrent writes)
- Inspiration: Redis (function memoization)

## Notes

This is a **foundational change** that affects nearly every part of ReedCMS. While risky, it solves fundamental limitations and sets ReedCMS up for enterprise scale.

The key insight: **CSV is structured data**, making row-level operations and intelligent merging possible (unlike Git's line-based text merging).
