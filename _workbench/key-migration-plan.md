# ReedCMS Key Migration Plan

**Date:** 2025-10-07
**Status:** Ready for Execution

---

## Migration Summary

- **Total Keys:** 547 unique base keys
- **Files Affected:** `.reed/text.csv` (1073 entries), `.reed/meta.csv` (76 entries)
- **Templates Affected:** All `.jinja` files referencing migrated keys
- **Routes:** Already correct (no changes needed)

---

## New Key Naming Rules

### Grundregel
**Bindestriche innerhalb von Scopes, Punkte zwischen Scopes**

### Pattern-Hierarchie

**1. KOMPONENTEN (Organisms/Molecules/Atoms)**
```
{component-name}.{element}.{property}
```
Examples:
- `page-header.logo.alt`
- `page-footer.copyright.text`
- `svg-icon.default.alt`

**2. KNOWLEDGE TERMS (Organism: knowledge-term)**
```
knowledge-term.{term-name}.{property}
```
Examples:
- `knowledge-term.actix-web.title`
- `knowledge-term.apache-license.description`
- `knowledge-term.lost-in-the-middle.pros.1`

**3. LAYOUTS (Seitenspezifische Sektionen)**
```
{layout-name}.{section}.{element}
```
Examples:
- `landing.hero.title`
- `impressum.info.address`
- `knowledge.hero.subtitle`

**4. GENERIC KEYS (Keine Komponente, kein Layout)**
```
{category}.{property}
```
Examples:
- `page.title`
- `page.meta.description`
- `knowledge.term.category.label` (generic label, not specific term)

---

## Migration Mappings

### Knowledge Terms (11 terms)
```
actix.web.*              → knowledge-term.actix-web.*
aeo.*                    → knowledge-term.aeo.*
agility.*                → knowledge-term.agility.*
ai.visibility.*          → knowledge-term.ai-visibility.*
apache.license.*         → knowledge-term.apache-license.*
apache.*                 → knowledge-term.apache.*
api.*                    → knowledge-term.api.*
bastille.*               → knowledge-term.bastille.*
geo.*                    → knowledge-term.geo.*
lost.in.the.middle.*     → knowledge-term.lost-in-the-middle.*
seo.*                    → knowledge-term.seo.*
```

### Components (3 types)
```
page.header.*            → page-header.*
page.footer.*            → page-footer.*
svg.icon.*               → svg-icon.*
```

### Layouts (Unchanged)
```
landing.*                → landing.* (no change)
impressum.*              → impressum.* (no change)
knowledge.*              → knowledge.* (no change)
portfolio.*              → portfolio.* (no change)
blog.*                   → blog.* (no change)
contact.*                → contact.* (no change)
```

### Generic Keys (Unchanged)
```
page.title               → page.title (no change)
page.meta.*              → page.meta.* (no change)
page.description         → page.description (no change)
page.placeholder.*       → page.placeholder.* (no change)
page.skip.*              → page.skip.* (no change)
knowledge.term.*         → knowledge.term.* (no change - generic labels)
```

---

## Execution Plan

### Phase 1: CSV Migration
1. Backup current CSV files
2. Migrate `.reed/text.csv` (1073 entries)
3. Migrate `.reed/meta.csv` (76 entries)
4. Verify no duplicate keys created

### Phase 2: Template Updates
1. Find all templates using old keys
2. Update templates with new keys
3. Test template compilation

### Phase 3: Add Missing Entries
1. Add `knowledge-term.{name}@lang` entries for index display
2. Verify all index terms have entries

### Phase 4: Testing
1. Start server
2. Test all pages (landing, knowledge, portfolio, blog, impressum, contact)
3. Verify knowledge index displays all terms
4. Verify knowledge term detail pages work

---

## Additional Tasks

### New Index Entries Needed
All knowledge terms need index display entries:
```csv
knowledge-term.actix-web@de|Actix-Web|Index display
knowledge-term.actix-web@en|Actix-Web|Index display
knowledge-term.aeo@de|AEO|Index display
knowledge-term.aeo@en|AEO|Index display
...
```

---

## Rollback Plan

If migration fails:
1. Restore CSV backups from `.reed/*.csv.backup`
2. Revert template changes from git
3. Restart server

---

## Success Criteria

✅ All 1149 CSV entries migrated successfully
✅ No broken template references
✅ All pages render correctly
✅ Knowledge index shows all terms
✅ Knowledge term detail pages work
✅ No console errors

