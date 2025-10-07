# Migration Guides

Upgrade paths and migration procedures for ReedCMS.

## Version Migration

### General Migration Process

```bash
# 1. Backup everything
reed data:backup --all
reed data:export --format=json --output=backup.json

# 2. Review changes
reed migrate:check --from=0.1.0 --to=0.2.0

# 3. Test migration (dry-run)
reed migrate:run --from=0.1.0 --to=0.2.0 --dry-run

# 4. Run migration
reed migrate:run --from=0.1.0 --to=0.2.0

# 5. Verify
reed data:verify --all
reed server:start --test
```

## CSV Format Migrations

### Key Nomenclature Changes

**Scenario**: Renaming keys whilst preserving values

```bash
# Example: knowledge.title → kb.title

# 1. Backup first
reed data:backup text

# 2. Dry-run to preview
reed data:rename \
  --from="knowledge" \
  --to="kb" \
  --recursive \
  --dry-run

# Output:
#   knowledge.title@en → kb.title@en
#   knowledge.intro@en → kb.intro@en
#   knowledge.title@de → kb.title@de
#   ...

# 3. Execute rename
reed data:rename \
  --from="knowledge" \
  --to="kb" \
  --recursive

# 4. Update templates
find templates/ -name "*.jinja" -exec \
  sed -i 's/text("knowledge\./text("kb./g' {} \;

# 5. Verify
reed data:get kb.title@en
```

### Language Code Changes

**Scenario**: `en` → `en-GB`, `de` → `de-DE`

```bash
# Custom migration script
reed migrate:language-codes \
  --from=en --to=en-GB \
  --from=de --to=de-DE \
  --dry-run

# Output:
#   knowledge.title@en → knowledge.title@en-GB
#   knowledge.title@de → knowledge.title@de-DE
#   ...

# Execute
reed migrate:language-codes \
  --from=en --to=en-GB \
  --from=de --to=de-DE
```

### CSV Structure Changes

**Scenario**: Adding new column to CSV format

**Old Format**:
```
key|value
knowledge.title@en|Knowledge Base
```

**New Format**:
```
key|value|comment
knowledge.title@en|Knowledge Base|English page title
```

**Migration**:
```bash
# Automatic migration (adds empty comments)
reed migrate:csv-structure \
  --file=text \
  --add-column=comment \
  --default=""

# Or manual with script
for file in .reed/*.csv; do
    sed -i '1s/$/|comment/' "$file"
    sed -i '2,$s/$/|/' "$file"
done
```

## Data Structure Migrations

### Splitting CSV Files

**Scenario**: Split `text.csv` into `text.csv` and `meta.csv`

```bash
# 1. Create meta.csv
reed data:init meta

# 2. Move keys matching pattern
reed data:move \
  --from-file=text \
  --to-file=meta \
  --pattern="*.meta.*" \
  --dry-run

# Example moves:
#   knowledge.meta.title@en → meta.csv
#   knowledge.meta.description@en → meta.csv
#   ...

# 3. Execute
reed data:move \
  --from-file=text \
  --to-file=meta \
  --pattern="*.meta.*"

# 4. Update code references
# Change: reedbase.text()
# To: reedbase.meta()
```

### Merging CSV Files

**Scenario**: Merge `routes_de.csv` and `routes_en.csv` into `routes.csv`

```bash
# 1. Export both
reed data:export --file=routes_de --output=routes_de.json
reed data:export --file=routes_en --output=routes_en.json

# 2. Merge (custom script)
reed migrate:merge-csv \
  --files=routes_de.csv,routes_en.csv \
  --output=routes.csv \
  --add-language-suffix

# 3. Verify keys
reed data:list routes | wc -l
# Should equal: routes_de + routes_en count

# 4. Remove old files
rm .reed/routes_de.csv .reed/routes_en.csv
```

## Template Migrations

### Filter Renaming

**Scenario**: `get_text()` → `text()`

```bash
# 1. Find all usage
grep -r "get_text(" templates/

# 2. Replace (dry-run)
find templates/ -name "*.jinja" -exec \
  grep -l "get_text(" {} \; | \
  xargs -I {} echo "Would update: {}"

# 3. Execute replacement
find templates/ -name "*.jinja" -exec \
  sed -i 's/get_text(/text(/g' {} \;

# 4. Verify (should be zero)
grep -r "get_text(" templates/ | wc -l
```

### Component Structure Changes

**Scenario**: Atomic Design reorganisation

**Old Structure**:
```
templates/components/
├── header.jinja
├── footer.jinja
└── nav.jinja
```

**New Structure**:
```
templates/components/
├── atoms/
│   └── icon/
├── molecules/
│   └── nav-item/
└── organisms/
    ├── page-header/
    └── page-footer/
```

**Migration**:
```bash
# 1. Backup templates
tar -czf templates-backup.tar.gz templates/

# 2. Create new structure
mkdir -p templates/components/{atoms,molecules,organisms}

# 3. Move components
mv templates/components/header.jinja \
   templates/components/organisms/page-header/page-header.mouse.jinja

# 4. Update includes in layouts
find templates/layouts/ -name "*.jinja" -exec \
  sed -i 's|{% include "components/header.jinja" %}|{% include "components/organisms/page-header/page-header.jinja" %}|g' {} \;

# 5. Rebuild assets
reed build:assets
```

## Server Configuration Migrations

### Port Changes

**Scenario**: Move from port 8333 (dev) to 3000 (production)

```bash
# Update server.csv
reed config:set server.port 3000

# Or with environment override
reed config:set server.port@prod 80
reed config:set server.port@dev 8333

# Restart server
systemctl restart reedcms
```

### Host Binding Changes

**Scenario**: `127.0.0.1` → `0.0.0.0`

```bash
# WARNING: Security implication!
# Only bind to 0.0.0.0 if behind firewall

reed config:set server.host 0.0.0.0
systemctl restart reedcms
```

## Security Migrations

### Password Hash Algorithm Update

**Scenario**: Upgrade Argon2 parameters

```bash
# New parameters in security.csv
reed config:set security.argon2.memory 32768  # 32MB (was 19456)
reed config:set security.argon2.iterations 3  # 3 passes (was 2)

# Re-hash all passwords on next login
# (automatic - users re-hashed at next auth)

# Or force re-hash all
reed security:rehash-passwords --force
```

### Permission System Changes

**Scenario**: Simple permissions → RBAC

**Old**:
```csv
user.permissions@admin|read,write,delete
```

**New**:
```csv
role.admin@system|users[rwx],content[rwx],config[rwx]
user.role@admin|admin
```

**Migration**:
```bash
# 1. Create roles from permissions
reed migrate:permissions-to-rbac --dry-run

# 2. Execute
reed migrate:permissions-to-rbac

# 3. Assign roles to users
reed user:role admin@example.com admin
```

## Asset Migrations

### Session Hash Regeneration

**Scenario**: Force cache invalidation

```bash
# Regenerate session hash
reed assets:hash

# Rebuild all assets with new hash
reed build:assets

# Old bundles automatically cleaned
```

### Asset Structure Changes

**Scenario**: `public/css/` → `public/session/styles/`

```bash
# 1. Rebuild with new structure
reed build:assets

# 2. Update nginx config
# Old: location /css/
# New: location /session/styles/

# 3. Update templates (automatic via manifest)
# No changes needed - templates use asset() helper
```

## Database Migrations

### CSV to Database Migration

**Scenario**: Moving from CSV to PostgreSQL (future)

```bash
# 1. Export all CSV data
reed data:export --all --format=json --output=export.json

# 2. Transform to SQL
reed migrate:csv-to-sql \
  --input=export.json \
  --output=schema.sql \
  --database=postgresql

# 3. Import to database
psql -U reedcms -d reedcms -f schema.sql

# 4. Update configuration
reed config:set database.enabled true
reed config:set database.url "postgresql://..."

# 5. Restart with database support
systemctl restart reedcms
```

## Rollback Procedures

### CSV Rollback

```bash
# List available backups
reed data:backup --list

# Output:
#   text.csv.20250115-143218.xz (2.4 MB)
#   text.csv.20250115-102341.xz (2.3 MB)
#   ...

# Restore specific backup
reed data:restore text.csv.20250115-143218.xz

# Or restore latest
reed data:restore --latest
```

### Full System Rollback

```bash
# 1. Stop server
systemctl stop reedcms

# 2. Restore all data
for backup in .reed/backups/*.xz; do
    reed data:restore "$backup"
done

# 3. Restore binary (if updated)
cp /opt/reedcms/backup/reedcms /opt/reedcms/reedcms

# 4. Restart server
systemctl start reedcms
```

### Git-Based Rollback

```bash
# If using version control

# 1. Check recent commits
git log --oneline -10

# 2. Revert to specific commit
git revert <commit-hash>

# 3. Rebuild assets
reed build:assets

# 4. Restart
systemctl restart reedcms
```

## Testing Migrations

### Migration Test Checklist

```bash
# 1. Data integrity
reed data:verify --all

# 2. Key existence
reed data:list | wc -l  # Should match expected count

# 3. Template rendering
for layout in landing knowledge blog; do
    reed debug:route "/$layout" || echo "FAIL: $layout"
done

# 4. Asset availability
for file in public/session/styles/*.css; do
    test -f "$file" || echo "FAIL: $file"
done

# 5. Server startup
reed server:start --test

# 6. HTTP endpoints
curl -I http://localhost:3000/
curl -I http://localhost:3000/knowledge
curl -I http://localhost:3000/health
```

## Common Migration Scenarios

### Adding New Language

```bash
# 1. Add language to project
reed config:set project.languages "en,de,fr"

# 2. Copy existing language as template
reed data:copy-language \
  --from=en \
  --to=fr \
  --mark-as-todo

# 3. Translate marked entries
reed data:list --filter=TODO | while read key; do
    reed data:set "$key" "TODO: Translate from EN"
done

# 4. Create route mappings
reed data:set landing@fr "" "French homepage"
reed data:set knowledge@fr "savoir" "French knowledge section"

# 5. Rebuild assets
reed build:assets

# 6. Test
reed debug:route /fr/
reed debug:route /fr/savoir
```

### Renaming Layout

```bash
# Old: knowledge
# New: kb

# 1. Rename template directory
mv templates/layouts/knowledge templates/layouts/kb

# 2. Rename template files
cd templates/layouts/kb
for file in knowledge.*; do
    mv "$file" "${file/knowledge/kb}"
done

# 3. Update routes
reed data:set kb@en "kb" "Knowledge base (renamed)"
reed data:set kb@de "kb" "German KB (renamed)"
reed data:delete knowledge@en
reed data:delete knowledge@de

# 4. Update registry
reed config:set layouts.available "landing,kb,blog,portfolio"

# 5. Rebuild
reed build:assets

# 6. Test
reed debug:route /kb
```

## Migration Best Practices

### Pre-Migration

1. **Full Backup**: Always backup before migration
2. **Dry-Run**: Test migration with `--dry-run` first
3. **Documentation**: Document custom migrations
4. **Downtime**: Schedule maintenance window
5. **Rollback Plan**: Prepare rollback procedure

### During Migration

1. **Monitor**: Watch logs during migration
2. **Verify**: Check each step completes successfully
3. **Incremental**: Migrate in small steps, not all at once
4. **Test**: Test after each major step

### Post-Migration

1. **Verification**: Run full system tests
2. **Performance**: Check performance hasn't degraded
3. **Monitoring**: Watch metrics for anomalies
4. **Documentation**: Update migration log
5. **Cleanup**: Remove old files after verification period

## See Also

- [CSV File Formats](csv-file-formats.md) - CSV specifications
- [Data Operations](../02-data-layer/data-operations.md) - Data manipulation
- [Backup System](../02-data-layer/backup-system.md) - Backup/restore
- [CLI Commands](../04-cli-layer/README.md) - Command reference
