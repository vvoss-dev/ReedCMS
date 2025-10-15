# Metrics Implementation Plan - QS Checklist

## Status: ✅ COMPLETED
**Created:** 2025-10-15  
**Completed:** 2025-10-15  
**Goal:** Consistent, KISS-compliant metrics across all 19 REED-19 tickets

---

## Design Principles

### 1. KISS - Keep It Simple, Stupid
- ❌ NO copy-paste code in every ticket
- ✅ ONE reusable metrics module
- ✅ Tickets only define WHAT to measure, not HOW

### 2. DRY - Don't Repeat Yourself
- ❌ NO duplicate MetricsProvider implementations
- ✅ Standard trait in `reedstream.rs` or `metrics/mod.rs`
- ✅ Tickets use declarative metric definitions

### 3. Consistency
- ✅ Every ticket has IDENTICAL section structure
- ✅ Every ticket has IDENTICAL collection strategy format
- ✅ Every metric follows SAME naming convention

---

## Proposed Architecture

### Reusable Metrics Module

```
src/reedcms/reedbase/metrics/
├── mod.rs                    # Public exports
├── collector.rs              # MetricsCollector implementation
├── types.rs                  # Metric, MetricDefinition, AlertRule types
├── storage.rs                # CSV storage backend
└── aggregator.rs             # Percentile/aggregation logic
```

### Usage in Tickets (Declarative)

```rust
// In each module (e.g., reedbase/tables/table.rs)
use crate::metrics::{MetricsProvider, MetricDefinition, AlertRule};

impl MetricsProvider for Table {
    fn metrics_spec() -> Vec<MetricDefinition> {
        vec![
            metric!("table_read_latency", Histogram, Microseconds, 1000)
                .with_target(1000.0)
                .with_alert(p99_exceeds(5000.0, "5min")),
        ]
    }
}

// Automatic instrumentation via derive macro (future improvement)
#[derive(Metrics)]
impl Table {
    #[measure("table_read_latency")]
    pub fn read_current(&self) -> ReedResult<Vec<u8>> {
        // Implementation
    }
}
```

---

## Standard Section Template (All Tickets)

```markdown
## Metrics & Observability

### Performance Metrics

| Metric | Type | Unit | Target | P99 Alert | Collection Point |
|--------|------|------|--------|-----------|------------------|
| {name} | {type} | {unit} | {target} | {alert} | {file}:{function} |

### Alert Rules

**CRITICAL Alerts:**
- `{metric} {condition}` for {duration} → "{message}"

**WARNING Alerts:**
- `{metric} {condition}` for {duration} → "{message}"

### Implementation

```rust
impl MetricsProvider for {Module} {
    fn metrics_spec() -> Vec<MetricDefinition> {
        vec![
            // Declarative definitions only
        ]
    }
}
```

### Collection Strategy

- **Sampling**: {All operations | 10% sample | Per request}
- **Aggregation**: {1-minute | 5-minute} rolling window
- **Storage**: `.reedbase/metrics/{module}.csv`
- **Retention**: 7 days raw, 90 days aggregated

### Why These Metrics Matter

**{Primary Metric}**: {Explanation}
- {Detail 1}
- {Detail 2}
- {Impact on system}
```

---

## QS Checklist Per Ticket

### Ticket Completion Criteria

- [ ] **Metrics Table**: Minimum 5, maximum 12 metrics
- [ ] **Alert Rules**: At least 2 CRITICAL, 2 WARNING
- [ ] **Implementation**: Uses `MetricsProvider` trait
- [ ] **Collection Strategy**: ALL 4 fields present (Sampling, Aggregation, Storage, Retention)
- [ ] **Why These Metrics Matter**: 3-5 key explanations
- [ ] **Code Examples**: Rust code compiles (checked mentally for syntax)
- [ ] **Naming Convention**: `{module}_{operation}_{metric_type}` (e.g., `table_read_latency`)
- [ ] **Consistency**: Matches pattern from REED-19-01 exactly

---

## Ticket Progress Tracker

### Infrastructure Ticket
- [x] REED-19-01A - Metrics Infrastructure (✓ Complete - reusable module)

### All 19 Core Tickets
- [x] REED-19-01 - Registry Dictionary (✓ Complete)
- [x] REED-19-02 - Universal Table API (✓ Complete)
- [x] REED-19-03 - Binary Delta Versioning (✓ Complete)
- [x] REED-19-04 - Crash Recovery (✓ Complete)
- [x] REED-19-05 - Concurrent Write System (✓ Complete)
- [x] REED-19-06 - Row-Level Merge (✓ Complete)
- [x] REED-19-07 - Conflict Resolution (✓ Complete)
- [x] REED-19-08 - Schema Validation (✓ Complete)
- [x] REED-19-09 - Column Schema Validation (✓ Complete)
- [x] REED-19-10 - Function System & Caching (✓ Complete)
- [x] REED-19-11 - Smart Indices (✓ Complete)
- [x] REED-19-12 - ReedQL Parser (✓ Complete)
- [x] REED-19-13 - Migration from REED-02 (✓ Complete)
- [x] REED-19-14 - Performance Testing (✓ Complete)
- [x] REED-19-15 - Documentation (✓ Complete)
- [x] REED-19-16 - Database Registry (✓ Complete)
- [x] REED-19-17 - Multi-Location Sync (✓ Complete)
- [x] REED-19-18 - P2P Routing (✓ Complete)
- [x] REED-19-19 - Installation Certificates (✓ Complete)

### QS Verification
- [x] REED-19-01: All 4 Collection Strategy fields present ✓
- [x] REED-19-10: All 4 Collection Strategy fields present ✓
- [x] REED-19-19: All 4 Collection Strategy fields present ✓
- [x] Consistent structure across all tickets ✓
- [x] Consistent naming convention ✓
- [x] KISS implementation (single-line metrics().record()) ✓

---

## Implementation Order

### Step 1: Design Reusable Module (30 min)
- Define `MetricsProvider` trait in `reedstream.rs` or `metrics/mod.rs`
- Create standard types: `MetricDefinition`, `AlertRule`, `MetricType`, `MetricUnit`
- Document in `_workbench/Tickets/templates/metrics-template.md`

### Step 2: Fix Existing Tickets (20 min)
- REED-19-02: Add missing Collection Strategy fields
- REED-19-05: Add missing Collection Strategy fields
- Verify consistency with REED-19-01

### Step 3: QS Review Agent Tickets (30 min)
- Review REED-19-03, 04, 06, 07, 08
- Check Collection Strategy presence
- Fix any inconsistencies

### Step 4: Complete Remaining Tickets (2 hours)
- REED-19-09 through REED-19-19
- Use QS checklist for each
- Commit after every 3 tickets

### Step 5: Final Consistency Review (30 min)
- Read all 19 tickets
- Verify naming conventions
- Verify section structure
- Single commit with all changes

**Total Estimated Time:** 3.5 hours

---

## Success Criteria

- [x] All 19 tickets have Metrics & Observability section ✓
- [x] All sections follow IDENTICAL structure ✓
- [x] All Collection Strategy blocks have 4 fields (Sampling, Aggregation, Storage, Retention) ✓
- [x] All metric names follow convention: `{module}_{operation}_{type}` ✓
- [x] No copy-paste code - all use reusable metrics module (REED-19-01A) ✓
- [x] REED-19-01A infrastructure ticket created ✓
- [ ] Final commit ready: `[REED-19-00][REED-19-01A] – feat: complete metrics implementation across all tickets`

---

## Notes

**Key Insight from User Feedback:**
> "das system musst super stringent überall gleich funktionieren und andocket - gleiche syntax, gleiche logik"

This means:
- NO variations in section structure
- NO "creative" implementations per ticket
- YES to boring, consistent, predictable patterns
- YES to maximizing code reuse

**CLAUDE.md Rule 0:**
> NEVER duplicate existing functions - use existing, extend if needed

This applies to metrics too:
- Don't write new metric collection code per ticket
- Use ONE shared MetricsCollector implementation
- Tickets only define WHAT to measure (declarative)
