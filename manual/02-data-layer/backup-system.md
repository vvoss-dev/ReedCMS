# Backup System

> Automatic XZ-compressed backups with 32-backup retention

---

## Overview

Every write operation automatically creates an XZ-compressed backup before modifying CSV files, ensuring data safety and rollback capability.

---

## Backup Architecture

```
┌──────────────────────────────────────────────────┐
│         Write Operation (set_text)               │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│       1. Read Current CSV File                   │
│          .reed/text.csv                          │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│       2. Compress with XZ (LZMA2)                │
│          Compression ratio: ~10:1                │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│       3. Save to Backups Directory               │
│          .reed/backups/text.csv.{timestamp}.xz   │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│       4. Delete Oldest if > 32 Backups           │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│       5. Write New CSV File (Atomic)             │
│          temp.csv → text.csv (rename)            │
└──────────────────────────────────────────────────┘
```

---

## Backup Format

### File Naming

```
{filename}.{unix_timestamp}.csv.xz
```

**Examples:**
```
text.csv.1704067200.csv.xz
routes.csv.1704067215.csv.xz
meta.csv.1704067230.csv.xz
```

**Timestamp:** Unix epoch seconds (sortable, unique)

### Location

```
.reed/backups/
├── text.csv.1704067200.csv.xz
├── text.csv.1704067140.csv.xz
├── text.csv.1704067080.csv.xz
├── routes.csv.1704067200.csv.xz
└── ...
```

**Directory:** `.reed/backups/` (created automatically)

### Compression

**Algorithm:** XZ (LZMA2)

**Compression ratio:**
- Text-heavy content: ~10:1
- Mixed content: ~5:1
- Already compressed data: ~1:1

**Example:**
```
Original: text.csv (100 KB)
Compressed: text.csv.{timestamp}.xz (10 KB)
```

---

## Retention Policy

**Maximum backups:** 32 per file

**Deletion strategy:** FIFO (First In, First Out)

**Example:**
```
33rd backup created
→ Oldest backup (1st) deleted automatically
→ Total remains at 32
```

**Calculation:**
```
1 write/hour × 24 hours × 32 backups = 32 days history
10 writes/hour × 24 hours × 32 backups = 3.2 days history
```

**Adjust retention:** Modify `MAX_BACKUPS` constant in code

---

## Backup Operations

### Automatic Backup

**Triggered by:** Any CSV write operation

**CLI commands that trigger backups:**
```bash
reed text:set page.title@en "Welcome"      # Backs up text.csv
reed route:set knowledge@de "wissen"       # Backs up routes.csv
reed meta:set site.title "ReedCMS"         # Backs up meta.csv
reed user:create admin --email ...         # Backs up users.matrix.csv
reed role:create editor --permissions ...  # Backs up roles.matrix.csv
```

**Process:**
1. Read current CSV → 2. Compress to XZ → 3. Save backup → 4. Write new CSV

**Performance:** < 20ms additional overhead per write

### Manual Backup

**Export to JSON (recommended):**
```bash
# Complete system backup
reed text:export backup-text-$(date +%Y%m%d).json
reed route:export backup-routes-$(date +%Y%m%d).json
reed meta:export backup-meta-$(date +%Y%m%d).json
reed user:export backup-users-$(date +%Y%m%d).json
reed role:export backup-roles-$(date +%Y%m%d).json
```

**Export to CSV:**
```bash
# Copy files directly
cp .reed/text.csv backups/text-$(date +%Y%m%d).csv
cp .reed/routes.csv backups/routes-$(date +%Y%m%d).csv
```

---

## Restore Operations

### List Available Backups

```bash
# Show all backups
ls -lh .reed/backups/

# Show backups for specific file
ls -lh .reed/backups/text.csv.*

# Show 5 most recent
ls -lt .reed/backups/text.csv.* | head -5
```

### Restore from Backup

**1. Decompress backup:**
```bash
# Find backup timestamp
ls .reed/backups/text.csv.*

# Decompress
xz -d < .reed/backups/text.csv.1704067200.csv.xz > text-restored.csv
```

**2. Verify content:**
```bash
# Check first few lines
head -20 text-restored.csv

# Count records
wc -l text-restored.csv
```

**3. Replace current file:**
```bash
# Backup current state first!
cp .reed/text.csv .reed/text.csv.before-restore

# Replace with restored version
mv text-restored.csv .reed/text.csv

# Restart server to reload cache
reed server:restart
```

### Restore Specific Entry

**Extract single key from backup:**
```bash
# Decompress and grep
xz -d < .reed/backups/text.csv.1704067200.csv.xz | grep "page.title@en"

# Output: page.title@en|Old Value|Description
```

**Manually restore:**
```bash
reed text:set page.title@en "Old Value" --desc "Description"
```

---

## Backup Storage

### Disk Space Usage

**Per backup:** ~10 KB (for 100 KB CSV with text content)

**32 backups:** ~320 KB

**All CSV files (5 files × 32 backups):** ~1.6 MB

**Recommendation:** Monitor if CSV files grow beyond 1 MB

### Cleanup

**Automatic:** Handled by retention policy (32 backups)

**Manual cleanup:**
```bash
# Delete backups older than 30 days
find .reed/backups/ -name "*.xz" -mtime +30 -delete

# Keep only 10 most recent per file
for file in text routes meta; do
    ls -t .reed/backups/${file}.csv.* | tail -n +11 | xargs rm -f
done
```

---

## Performance Impact

### Write Operations

| Operation | Without Backup | With Backup | Overhead |
|-----------|----------------|-------------|----------|
| Small file (1 KB) | ~5ms | ~15ms | +10ms |
| Medium file (10 KB) | ~10ms | ~25ms | +15ms |
| Large file (100 KB) | ~20ms | ~40ms | +20ms |

**Overhead:** ~10-20ms per write (acceptable for most use cases)

### Compression Performance

| File Size | Compression Time | Output Size | Ratio |
|-----------|------------------|-------------|-------|
| 1 KB | ~2ms | ~200 bytes | 5:1 |
| 10 KB | ~5ms | ~1 KB | 10:1 |
| 100 KB | ~20ms | ~10 KB | 10:1 |

**XZ settings:** Default compression level (6)

---

## Best Practices

**Don't commit backups to Git:**
```.gitignore
.reed/backups/
```

**Regular off-site backups:**
```bash
# Daily cron job
0 2 * * * cd /path/to/reedcms && tar czf ~/backups/reedcms-$(date +\%Y\%m\%d).tar.gz .reed/
```

**Test restore procedures:**
```bash
# Monthly drill
xz -d < .reed/backups/text.csv.latest.xz > /tmp/test-restore.csv
diff .reed/text.csv /tmp/test-restore.csv
```

**Monitor backup directory size:**
```bash
# Check size
du -sh .reed/backups/

# Alert if > 10 MB
if [ $(du -sm .reed/backups/ | cut -f1) -gt 10 ]; then
    echo "Backup directory large!"
fi
```

**Adjust retention for high-write scenarios:**
```rust
// src/reedcms/reedbase/set.rs
const MAX_BACKUPS: usize = 64; // Increase from 32
```

---

## Disaster Recovery

### Full System Restore

**1. Restore .reed/ directory from off-site backup:**
```bash
tar xzf reedcms-backup.tar.gz
```

**2. Verify CSV files:**
```bash
head .reed/text.csv
head .reed/routes.csv
head .reed/meta.csv
```

**3. Restart system:**
```bash
reed server:restart
```

### Partial Restore (Single File)

**1. Find backup timestamp:**
```bash
ls -lt .reed/backups/text.csv.* | head -5
```

**2. Decompress:**
```bash
xz -d < .reed/backups/text.csv.1704067200.csv.xz > /tmp/text-restored.csv
```

**3. Compare with current:**
```bash
diff .reed/text.csv /tmp/text-restored.csv
```

**4. Restore if needed:**
```bash
cp .reed/text.csv .reed/text.csv.backup
mv /tmp/text-restored.csv .reed/text.csv
reed server:restart
```

---

## Monitoring

### Backup Health Check

```bash
# Check last backup time
ls -lt .reed/backups/*.xz | head -1

# Verify can decompress
xz -t .reed/backups/*.xz 2>&1 | grep -v "OK" || echo "All backups valid"

# Count backups per file
for file in text routes meta users roles; do
    count=$(ls .reed/backups/${file}.*.xz 2>/dev/null | wc -l)
    echo "${file}: ${count} backups"
done
```

### Automated Monitoring

```bash
#!/bin/bash
# backup-monitor.sh

# Check backup directory exists
if [ ! -d .reed/backups ]; then
    echo "ERROR: Backup directory missing!"
    exit 1
fi

# Check recent backups
for file in text routes meta; do
    latest=$(ls -t .reed/backups/${file}.csv.*.xz 2>/dev/null | head -1)
    if [ -z "$latest" ]; then
        echo "WARNING: No backups for ${file}.csv"
    else
        age=$(($(date +%s) - $(stat -f%m "$latest")))
        if [ $age -gt 86400 ]; then
            echo "WARNING: ${file}.csv backup older than 24h"
        fi
    fi
done

echo "Backup check complete"
```

---

**See also:**
- [CSV Architecture](csv-architecture.md) - File format details
- [ReedBase Cache](reedbase-cache.md) - Cache and write operations
- [Data Operations](data-operations.md) - API reference
