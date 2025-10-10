# ReedCMS Implementation Roadmap
**Last Updated**: 2025-10-10  
**Purpose**: Logical implementation sequence for remaining tickets

---

## Summary of Missing Tickets

**Total Missing**: 11 tickets across 3 categories

### Category 1: Data Layer Completion (1 ticket)
- **REED-02-03**: Environment Fallback System

### Category 2: Monitor Layer (4 tickets)
- **REED-10-01**: ReedMonitor Foundation
- **REED-10-02**: Performance Profiler
- **REED-10-03**: Debug Tools
- **REED-10-04**: Backup Recovery CLI

### Category 3: Extension Layer (4 tickets)
- **REED-11-01**: Hook System
- **REED-11-02**: Workflow Engine
- **REED-11-03**: External API Bridges
- **REED-11-04**: Scheduled Tasks

### Category 4: Quality (1 ticket)
- **REED-90-01**: Quality Standards Restoration

### Category 5: Third-Party (Planned - Not Critical)
- **REED-20-01**: MCP Server Library
- **REED-20-02**: VS Code Extension
- **REED-20-03**: Zed Extension
- **REED-20-04**: JetBrains Extension

---

## Dependency Analysis

### REED-02-03: Environment Fallback System
**Status**: Functionality EXISTS in REED-02-01  
**Dependencies**: None (REED-02-01 already complete)  
**Blocks**: Technically nothing (fallback already works)  
**Priority**: LOW - Documentation task only  
**Effort**: 1-2 hours (explicit function extraction + tests)

**Analysis**: The environment fallback logic (`key@env → key`) is already implemented in REED-02-01's ReedBase cache system. This ticket just needs explicit extraction into dedicated functions for clarity and testability.

---

### REED-10-01: ReedMonitor Foundation
**Dependencies**: 
- REED-06-01 (Server Foundation) ✅ Complete
**Blocks**: 
- REED-10-02 (Performance Profiler)
- REED-10-03 (Debug Tools)
**Priority**: HIGH - Foundation for all monitoring  
**Effort**: 8-12 hours

**Components**:
1. FreeBSD-style syslog logger (RFC 5424 levels)
2. Metrics collection system (requests, ReedBase, templates)
3. Actix-Web middleware integration
4. Health check endpoints (/health, /metrics)
5. Log file rotation and compression

**Critical for**: Production monitoring and debugging

---

### REED-10-02: Performance Profiler
**Dependencies**: 
- REED-10-01 (ReedMonitor Foundation) ❌ Not started
**Blocks**: 
- REED-10-03 (Debug Tools - uses profiler data)
**Priority**: MEDIUM  
**Effort**: 6-8 hours

**Components**:
1. Span-based profiler with nested tracking
2. Profiling middleware
3. Slow query tracker
4. Flame graph generator
5. CLI commands (profile:request, profile:slow, profile:flamegraph)

**Dependency Chain**: Must wait for REED-10-01

---

### REED-10-03: Debug Tools
**Dependencies**: 
- REED-10-01 (ReedMonitor) ❌ Not started
- REED-10-02 (Profiler) ❌ Not started
**Blocks**: None  
**Priority**: LOW - Development convenience  
**Effort**: 4-6 hours

**Components**:
1. Request inspector
2. Cache viewer
3. Route tester
4. Template debugger
5. Config inspector

**Dependency Chain**: Must wait for REED-10-01 + REED-10-02

---

### REED-10-04: Backup Recovery CLI
**Dependencies**: 
- REED-02-04 (Backup System) ✅ Complete
**Blocks**: None  
**Priority**: HIGH - Data safety critical  
**Effort**: 4-6 hours

**Components**:
1. List backups (backup:list)
2. Restore backup (backup:restore)
3. Verify backup (backup:verify)
4. Prune old backups (backup:prune)
5. Timestamp-based restore (backup:restore-timestamp)

**Critical for**: Disaster recovery and data protection

---

### REED-11-01: Hook System
**Dependencies**: None (independent)  
**Blocks**: REED-11-02 (Workflow Engine may use hooks)  
**Priority**: MEDIUM - Extension foundation  
**Effort**: 6-8 hours

**Analysis**: Event-driven hook system for extensibility. Independent of other Extension Layer tickets.

---

### REED-11-02: Workflow Engine
**Dependencies**: 
- REED-11-01 (Hook System) ❌ Not started (optional)
**Blocks**: None  
**Priority**: LOW - Advanced feature  
**Effort**: 10-15 hours

**Analysis**: Can work without hooks but benefits from hook integration.

---

### REED-11-03: External API Bridges
**Dependencies**: None (independent)  
**Blocks**: None  
**Priority**: LOW - Optional integration  
**Effort**: 8-12 hours

**Analysis**: External service integration (Stripe, SendGrid, etc.). Standalone implementation.

---

### REED-11-04: Scheduled Tasks
**Dependencies**: None (independent)  
**Blocks**: None  
**Priority**: MEDIUM - Automation feature  
**Effort**: 6-8 hours

**Analysis**: Cron-like task scheduler. Independent of other tickets.

---

### REED-90-01: Quality Standards Restoration
**Dependencies**: None  
**Blocks**: None  
**Priority**: MEDIUM - Code quality  
**Effort**: 4-8 hours

**Analysis**: Code review, refactoring, documentation improvements. Can run in parallel.

---

## Recommended Implementation Order

### Phase 1: Critical Foundation (Priority: HIGH)
**Objective**: Complete core functionality and data safety

#### 1.1. REED-02-03: Environment Fallback System
- **Effort**: 1-2 hours
- **Reason**: Quick win, clarifies existing functionality
- **Action**: Extract fallback logic into explicit functions
- **Files**: `src/reedcms/reedbase/environment.rs`

#### 1.2. REED-10-04: Backup Recovery CLI
- **Effort**: 4-6 hours
- **Reason**: Critical for data safety, independent of other Monitor tickets
- **Action**: Implement CLI commands for backup management
- **Files**: `src/reedcms/backup/recovery.rs`, `src/reedcms/cli/commands/backup.rs`

**Checkpoint**: Core system 100% complete, data safety guaranteed

---

### Phase 2: Monitoring Infrastructure (Priority: HIGH)
**Objective**: Production monitoring and observability

#### 2.1. REED-10-01: ReedMonitor Foundation
- **Effort**: 8-12 hours
- **Reason**: Foundation for all monitoring, unblocks REED-10-02 and REED-10-03
- **Action**: Implement syslog, metrics collection, middleware, health endpoints
- **Files**: 
  - `src/reedcms/monitor/syslog.rs`
  - `src/reedcms/monitor/metrics.rs`
  - `src/reedcms/monitor/core.rs`
  - `src/reedcms/monitor/middleware.rs`

#### 2.2. REED-10-02: Performance Profiler
- **Effort**: 6-8 hours
- **Reason**: Production performance analysis, unblocks REED-10-03
- **Action**: Implement span-based profiler, flame graphs, CLI commands
- **Files**: 
  - `src/reedcms/profiler/core.rs`
  - `src/reedcms/profiler/middleware.rs`
  - `src/reedcms/profiler/flamegraph.rs`

#### 2.3. REED-10-03: Debug Tools
- **Effort**: 4-6 hours
- **Reason**: Development productivity
- **Action**: Implement debug CLI commands (request, cache, route, config)
- **Files**: 
  - `src/reedcms/debug/request_inspector.rs`
  - `src/reedcms/debug/cache_viewer.rs`
  - `src/reedcms/debug/route_tester.rs`

**Checkpoint**: Full monitoring and debugging capabilities operational

---

### Phase 3: Extension Layer (Priority: MEDIUM)
**Objective**: Advanced extensibility features

#### 3.1. REED-11-01: Hook System
- **Effort**: 6-8 hours
- **Reason**: Foundation for event-driven extensions
- **Action**: Implement hook registration and execution system

#### 3.2. REED-11-04: Scheduled Tasks
- **Effort**: 6-8 hours
- **Reason**: Automation capability, independent
- **Action**: Implement cron-like task scheduler

#### 3.3. REED-11-03: External API Bridges
- **Effort**: 8-12 hours
- **Reason**: External service integration
- **Action**: Implement API bridge framework

#### 3.4. REED-11-02: Workflow Engine
- **Effort**: 10-15 hours
- **Reason**: Advanced automation, benefits from hooks
- **Action**: Implement workflow orchestration

**Checkpoint**: Full extension capabilities available

---

### Phase 4: Quality & Optimization (Priority: MEDIUM)
**Objective**: Code quality and maintainability

#### 4.1. REED-90-01: Quality Standards Restoration
- **Effort**: 4-8 hours
- **Reason**: Code review and refactoring
- **Action**: Review all code, apply CLAUDE.md standards, refactor violations

**Checkpoint**: Code quality standards enforced

---

### Phase 5: Third-Party Integrations (Priority: LOW - OPTIONAL)
**Objective**: Developer experience enhancements

These can be implemented independently when needed:
- REED-20-01: MCP Server Library
- REED-20-02: VS Code Extension
- REED-20-03: Zed Extension
- REED-20-04: JetBrains Extension

---

## Implementation Timeline Estimate

### Minimal Viable Implementation (Phases 1-2)
**Total Effort**: 24-36 hours (3-5 days of focused work)
- Phase 1: 5-8 hours (Data safety + quick fixes)
- Phase 2: 19-28 hours (Full monitoring stack)

**Result**: Production-ready system with complete monitoring

### Full Implementation (Phases 1-4)
**Total Effort**: 60-90 hours (8-12 days of focused work)
- Phase 1: 5-8 hours
- Phase 2: 19-28 hours
- Phase 3: 30-43 hours
- Phase 4: 4-8 hours

**Result**: Complete ReedCMS with all core and extension features

### Complete Implementation (Phases 1-5)
**Total Effort**: Variable (depends on IDE extension complexity)
- Phases 1-4: 60-90 hours
- Phase 5: 40-80 hours per extension

**Result**: Full ecosystem with IDE integrations

---

## Critical Decision Points

### Decision 1: Monitor Layer vs Extension Layer
**Question**: Implement monitoring first (REED-10-*) or extensions first (REED-11-*)?  
**Recommendation**: **Monitor Layer first**  
**Reasoning**:
- Monitor Layer is production-critical
- Provides visibility into system behaviour
- Essential for debugging Extension Layer features
- Lower risk, higher immediate value

### Decision 2: Full Monitor Layer vs Minimal
**Question**: Implement all monitor tickets or just REED-10-01 + REED-10-04?  
**Recommendation**: **Full Monitor Layer (all 4 tickets)**  
**Reasoning**:
- Strong dependencies between tickets
- REED-10-01 unblocks REED-10-02 and REED-10-03
- Profiler and debug tools have high development value
- Total effort difference: ~10 hours (24h vs 14h)

### Decision 3: Extension Layer Priority
**Question**: Which Extension Layer ticket first?  
**Recommendation**: **REED-11-01 (Hook System) → REED-11-04 (Scheduled Tasks)**  
**Reasoning**:
- Hook System is foundation for event-driven features
- Scheduled Tasks are immediately useful
- External API Bridges are use-case specific
- Workflow Engine is most complex, benefits from hooks

### Decision 4: Quality Standards Timing
**Question**: When to implement REED-90-01?  
**Recommendation**: **After Phase 2 (Monitor Layer complete)**  
**Reasoning**:
- New monitor code can be reviewed immediately
- Prevents accumulation of technical debt
- Can run in parallel with Extension Layer work
- Ensures high quality before final features

---

## Risk Assessment

### Low Risk Tickets ✅
- **REED-02-03**: Functionality exists, just documentation
- **REED-10-04**: Simple CLI, builds on proven backup system
- **REED-90-01**: Code review, no new features

### Medium Risk Tickets ⚠️
- **REED-10-01**: Complex but well-specified
- **REED-10-02**: Profiling overhead needs careful testing
- **REED-11-01**: Event system needs good design
- **REED-11-04**: Cron parsing can be tricky

### High Risk Tickets 🔴
- **REED-10-03**: Multiple integration points
- **REED-11-02**: Complex orchestration logic
- **REED-11-03**: External API reliability

---

## Parallel Implementation Strategy

If multiple developers are available, these can run in parallel:

### Track 1: Data & Monitoring
1. REED-02-03 (1-2h)
2. REED-10-04 (4-6h)
3. REED-10-01 (8-12h)
4. REED-10-02 (6-8h)
5. REED-10-03 (4-6h)

### Track 2: Extensions & Quality
1. REED-11-01 (6-8h) - can start early
2. REED-11-04 (6-8h)
3. REED-90-01 (4-8h) - review Track 1 code
4. REED-11-03 (8-12h)
5. REED-11-02 (10-15h)

**Total Time (Parallel)**: ~30-45 hours (vs 60-90 sequential)

---

## Next Steps

### Immediate Action (Start with Phase 1)

1. **REED-02-03**: Extract environment fallback functions
   - Read existing `src/reedcms/reed/reedbase.rs`
   - Extract fallback logic to `src/reedcms/reedbase/environment.rs`
   - Add tests
   - Commit: `[REED-02-03] – feat: extract environment fallback system`

2. **REED-10-04**: Implement backup recovery CLI
   - Create `src/reedcms/backup/recovery.rs`
   - Add CLI commands to `src/reedcms/cli/commands/backup.rs`
   - Add tests
   - Commit: `[REED-10-04] – feat: implement backup recovery CLI tools`

### User Approval Required

**Question for User**: Which implementation strategy do you prefer?

**Option A: Sequential (Recommended for solo developer)**
- Follow Phase 1 → Phase 2 → Phase 3 → Phase 4
- Clear progression, no conflicts
- 60-90 hours total

**Option B: Monitor Focus (Minimal viable)**
- Phase 1 + Phase 2 only
- Skip Extension Layer for now
- 24-36 hours total

**Option C: Parallel (If multiple developers)**
- Track 1 + Track 2 simultaneously
- 30-45 hours total
- Requires coordination

**Option D: Custom Priority**
- User specifies which tickets are most important
- We create custom sequence

---

## Commit Message Format

All commits MUST follow this format:
```
[REED-XX-YY] – type: short description

Optional longer description with details.
Can span multiple lines.

- Bullet points for changes
- More details if needed
```

**Examples**:
- `[REED-02-03] – feat: extract environment fallback system`
- `[REED-10-01] – feat: implement ReedMonitor foundation with syslog`
- `[REED-10-04] – feat: implement backup recovery CLI commands`

---

## Success Criteria

### Phase 1 Complete ✅
- [ ] Environment fallback functions documented
- [ ] Backup recovery CLI functional
- [ ] All tests passing
- [ ] Documentation updated

### Phase 2 Complete ✅
- [ ] ReedMonitor collecting metrics
- [ ] Health endpoints responding
- [ ] Profiler generating flame graphs
- [ ] Debug tools functional

### Phase 3 Complete ✅
- [ ] Hook system operational
- [ ] Scheduled tasks running
- [ ] API bridges working
- [ ] Workflow engine executing

### Phase 4 Complete ✅
- [ ] Code quality standards enforced
- [ ] All CLAUDE.md rules followed
- [ ] No duplicate functions
- [ ] Documentation complete

---

## Conclusion

The implementation roadmap provides a clear, logical sequence for completing ReedCMS. The dependency analysis shows:

1. **REED-02-03** is trivial (1-2h) - quick win
2. **REED-10-04** is independent and critical (4-6h) - data safety
3. **REED-10-01** is the foundation for monitoring (8-12h) - must come first
4. **REED-10-02** and **REED-10-03** depend on REED-10-01
5. **Extension Layer** is independent but benefits from monitoring
6. **Quality standards** should be applied continuously

**Recommendation**: Start with Phase 1 (REED-02-03 + REED-10-04), then implement full Monitor Layer (Phase 2), followed by Extension Layer (Phase 3) and Quality review (Phase 4).

Total effort for core completion (Phases 1-2): **24-36 hours**  
Total effort for full implementation (Phases 1-4): **60-90 hours**
