# REED-11-04: Scheduled Tasks (Cron-Style Scheduling)

## MANDATORY Development Standards

**CRITICAL**: Every implementation MUST follow these standards:

- **Language**: All documentation and code comments in BBC English
- **Principle**: KISS (Keep It Simple, Stupid)
- **File Naming**: One file = One responsibility
- **Functions**: One function = One distinctive job
- **Testing**: Separate test files as `{name}_test.rs`

## Ticket Information
- **ID**: REED-11-04
- **Title**: Scheduled Tasks (Cron-Style Scheduling)
- **Layer**: Extension Layer (REED-11)
- **Priority**: Low
- **Status**: Open
- **Complexity**: Medium
- **Dependencies**: REED-11-02 (Workflow Engine)

## Objective
Implement cron-compatible task scheduling for automated workflow execution and maintenance tasks.

## Use Cases

### Daily Content Backup
```csv
schedule_id|cron|workflow|parameters|status|last_run|next_run
daily-backup|0 2 * * *|backup-content|{}|active|2025-10-02T02:00:00|2025-10-03T02:00:00
```

### Weekly Reports
```csv
weekly-report|0 9 * * 1|generate-analytics|{}|active|2025-09-30T09:00:00|2025-10-07T09:00:00
```

### Hourly Social Media Check
```csv
social-sync|0 * * * *|sync-social-mentions|{}|active|2025-10-02T14:00:00|2025-10-02T15:00:00
```

## Configuration

**.reed/schedules.csv**
```csv
schedule_id|cron|workflow|parameters|enabled|timezone|last_run|next_run|created_by|created_at
daily-backup|0 2 * * *|backup-content|{}|true|Europe/Berlin|2025-10-02T02:00:00|2025-10-03T02:00:00|admin|2025-10-02T...
weekly-report|0 9 * * 1|analytics-report|format=pdf|true|Europe/Berlin|||admin|2025-10-02T...
```

**Fields:**
- `schedule_id`: Unique identifier
- `cron`: Cron expression (minute hour day month weekday)
- `workflow`: Workflow ID to execute
- `parameters`: JSON parameters for workflow
- `enabled`: true/false
- `timezone`: Timezone for scheduling (default: UTC)
- `last_run`, `next_run`: Execution tracking
- `created_by`, `created_at`: Audit trail

## Cron Expression Format

Standard cron syntax:
```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6) (Sunday to Saturday)
│ │ │ │ │
* * * * *
```

**Examples:**
- `0 2 * * *` - Every day at 02:00
- `0 */6 * * *` - Every 6 hours
- `0 9 * * 1` - Every Monday at 09:00
- `*/15 * * * *` - Every 15 minutes
- `0 0 1 * *` - First day of every month at 00:00

**Special values:**
- `@hourly` - `0 * * * *`
- `@daily` - `0 0 * * *`
- `@weekly` - `0 0 * * 0`
- `@monthly` - `0 0 1 * *`
- `@yearly` - `0 0 1 1 *`

## Implementation

### File Structure
```
src/reedcms/
├── extensions/
│   ├── scheduler/
│   │   ├── mod.rs
│   │   ├── engine.rs           # Scheduler engine
│   │   ├── cron.rs             # Cron parser
│   │   ├── executor.rs         # Task executor
│   │   └── registry.rs         # Schedule registry
```

### Scheduler Engine

```rust
// src/reedcms/extensions/scheduler/engine.rs

use crate::reedcms::extensions::workflows::execute_workflow;
use std::time::Duration;

/// Scheduler engine that runs in background.
pub struct SchedulerEngine {
    running: Arc<AtomicBool>,
    schedules: Arc<RwLock<Vec<Schedule>>>,
}

impl SchedulerEngine {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            schedules: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Starts the scheduler engine.
    pub fn start(&self) -> ReedResult<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(ReedError::ConfigError {
                component: "scheduler".to_string(),
                reason: "Scheduler already running".to_string(),
            });
        }
        
        self.running.store(true, Ordering::SeqCst);
        self.load_schedules()?;
        
        // Start background thread
        let running = Arc::clone(&self.running);
        let schedules = Arc::clone(&self.schedules);
        
        std::thread::spawn(move || {
            scheduler_loop(running, schedules);
        });
        
        Ok(())
    }
    
    /// Stops the scheduler engine.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    fn load_schedules(&self) -> ReedResult<()> {
        let schedule_path = PathBuf::from(".reed/schedules.csv");
        if !schedule_path.exists() {
            return Ok(());
        }
        
        let entries = read_csv(&schedule_path)?;
        let mut schedules = self.schedules.write().unwrap();
        
        for entry in entries {
            let schedule = Schedule::from_csv_record(&entry)?;
            if schedule.enabled {
                schedules.push(schedule);
            }
        }
        
        Ok(())
    }
}

/// Main scheduler loop.
fn scheduler_loop(
    running: Arc<AtomicBool>,
    schedules: Arc<RwLock<Vec<Schedule>>>,
) {
    while running.load(Ordering::SeqCst) {
        let now = chrono::Utc::now();
        
        // Check each schedule
        let schedules_to_run = {
            let schedules = schedules.read().unwrap();
            schedules
                .iter()
                .filter(|s| s.should_run(&now))
                .cloned()
                .collect::<Vec<_>>()
        };
        
        // Execute due schedules
        for schedule in schedules_to_run {
            execute_scheduled_task(&schedule);
            update_schedule_times(&schedule, &now);
        }
        
        // Sleep for 1 minute
        std::thread::sleep(Duration::from_secs(60));
    }
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: String,
    pub cron: CronExpression,
    pub workflow: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
    pub timezone: chrono_tz::Tz,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
}

impl Schedule {
    fn should_run(&self, now: &chrono::DateTime<chrono::Utc>) -> bool {
        if let Some(next_run) = self.next_run {
            now >= &next_run
        } else {
            // First run - check cron
            self.cron.matches(now)
        }
    }
}

fn execute_scheduled_task(schedule: &Schedule) {
    // Execute workflow
    match execute_workflow(&schedule.workflow, &WorkflowContext {
        trigger: format!("schedule:{}", schedule.id),
        key: None,
        value: None,
        user: "scheduler".to_string(),
    }) {
        Ok(result) => {
            log::info!("Scheduled task {} completed: {:?}", schedule.id, result);
        }
        Err(e) => {
            log::error!("Scheduled task {} failed: {}", schedule.id, e);
        }
    }
}

fn update_schedule_times(schedule: &Schedule, now: &chrono::DateTime<chrono::Utc>) {
    // Update last_run and calculate next_run
    let next_run = schedule.cron.next_occurrence(now);
    
    // Update in .reed/schedules.csv
    // (Implementation...)
}
```

### Cron Parser

```rust
// src/reedcms/extensions/scheduler/cron.rs

#[derive(Debug, Clone)]
pub struct CronExpression {
    minute: CronField,
    hour: CronField,
    day: CronField,
    month: CronField,
    weekday: CronField,
}

impl CronExpression {
    /// Parses cron expression from string.
    ///
    /// ## Examples
    /// ```rust
    /// let cron = CronExpression::parse("0 2 * * *")?;  // Daily at 02:00
    /// let cron = CronExpression::parse("*/15 * * * *")?;  // Every 15 minutes
    /// ```
    pub fn parse(expr: &str) -> ReedResult<Self> {
        // Handle special expressions
        let expr = match expr {
            "@hourly" => "0 * * * *",
            "@daily" => "0 0 * * *",
            "@weekly" => "0 0 * * 0",
            "@monthly" => "0 0 1 * *",
            "@yearly" => "0 0 1 1 *",
            other => other,
        };
        
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(ReedError::ValidationError {
                field: "cron".to_string(),
                value: expr.to_string(),
                constraint: "Expected 5 fields (minute hour day month weekday)".to_string(),
            });
        }
        
        Ok(Self {
            minute: CronField::parse(parts[0], 0, 59)?,
            hour: CronField::parse(parts[1], 0, 23)?,
            day: CronField::parse(parts[2], 1, 31)?,
            month: CronField::parse(parts[3], 1, 12)?,
            weekday: CronField::parse(parts[4], 0, 6)?,
        })
    }
    
    /// Checks if datetime matches cron expression.
    pub fn matches(&self, dt: &chrono::DateTime<chrono::Utc>) -> bool {
        self.minute.matches(dt.minute() as u32)
            && self.hour.matches(dt.hour() as u32)
            && self.day.matches(dt.day())
            && self.month.matches(dt.month())
            && self.weekday.matches(dt.weekday().num_days_from_sunday())
    }
    
    /// Calculates next occurrence after given datetime.
    pub fn next_occurrence(&self, after: &chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
        // Find next matching datetime
        // (Implementation with chrono...)
        *after + chrono::Duration::hours(1)
    }
}

#[derive(Debug, Clone)]
enum CronField {
    All,                        // *
    Specific(u32),              // 5
    Range(u32, u32),            // 1-5
    Step(Box<CronField>, u32),  // */15
    List(Vec<CronField>),       // 1,3,5
}

impl CronField {
    fn parse(field: &str, min: u32, max: u32) -> ReedResult<Self> {
        // Parse cron field
        // (Implementation...)
        Ok(CronField::All)
    }
    
    fn matches(&self, value: u32) -> bool {
        match self {
            CronField::All => true,
            CronField::Specific(v) => value == *v,
            CronField::Range(start, end) => value >= *start && value <= *end,
            CronField::Step(base, step) => {
                base.matches(value) && (value % step == 0)
            }
            CronField::List(fields) => fields.iter().any(|f| f.matches(value)),
        }
    }
}
```

## CLI Commands

```bash
# Add scheduled task
reed schedule:add daily-backup --cron "0 2 * * *" --workflow backup-content
reed schedule:add weekly-report --cron "@weekly" --workflow analytics-report

# List scheduled tasks
reed schedule:list [--format table|json]

# Show schedule details
reed schedule:show daily-backup

# Enable/disable schedule
reed schedule:enable daily-backup
reed schedule:disable daily-backup

# Run schedule manually
reed schedule:run daily-backup

# Remove schedule
reed schedule:remove daily-backup

# Show scheduler status
reed schedule:status
```

## Integration with Server

The scheduler runs as part of the ReedCMS server:

```bash
# Server automatically starts scheduler
reed server:start

# Scheduler status
reed server:status
# Output:
# Server: Running
# Scheduler: Running (3 active schedules)
```

## Testing Requirements

### Unit Tests
- [ ] Cron expression parsing
- [ ] Datetime matching
- [ ] Next occurrence calculation
- [ ] Schedule loading

### Integration Tests
- [ ] Full scheduler cycle
- [ ] Workflow execution
- [ ] Error handling
- [ ] Timezone handling

## Acceptance Criteria
- [ ] Cron parser working
- [ ] Scheduler engine functional
- [ ] Workflow integration working
- [ ] Timezone support working
- [ ] CLI commands functional
- [ ] All tests pass
- [ ] BBC English throughout

## Dependencies
- REED-11-02: Workflow engine for task execution
- External: chrono, chrono-tz for datetime/timezone handling

## Future Extensions
- Web UI for schedule management
- Email notifications on failure
- Schedule history/logs
- Retry failed executions
- Schedule templates
