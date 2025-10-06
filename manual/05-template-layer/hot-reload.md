# Hot-Reload System

> Instant template updates during development

---

## Overview

ReedCMS provides hot-reload for templates in development mode, allowing instant feedback when editing Jinja templates without server restart.

---

## Configuration

### Enable Hot-Reload

**Set environment in `.env`:**
```env
ENVIRONMENT=dev
```

**MiniJinja auto-reload:**
```rust
let env_mode = std::env::var("ENVIRONMENT").unwrap_or("prod".to_string());

let mut env = Environment::new();

if env_mode == "dev" {
    env.set_auto_reload(true);   // Enable hot-reload
} else {
    env.set_auto_reload(false);  // Disable (production)
}
```

---

## How It Works

### Development Mode (Hot-Reload)

```
┌──────────────────────────────────────────────────┐
│         Browser Request                          │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     MiniJinja: Check Template Modified          │
│     stat("template.jinja").modified_time         │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     Modified? → Recompile Template               │
│     Not Modified? → Use Cached Version           │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     Render Template                              │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     Return HTML                                  │
└──────────────────────────────────────────────────┘
```

**Performance:** ~5ms overhead for file stat check

### Production Mode (Cached)

```
┌──────────────────────────────────────────────────┐
│         Browser Request                          │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     MiniJinja: Use Cached Compilation            │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     Render Template (< 1ms)                      │
└───────────────────┬──────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│     Return HTML                                  │
└──────────────────────────────────────────────────┘
```

**No file checks** - templates compiled once at startup.

---

## Development Workflow

### Start Server in Dev Mode

```bash
# Ensure ENVIRONMENT=dev in .env
echo "ENVIRONMENT=dev" > .env

# Start server
reed server:io
```

**Server output:**
```
🚀 ReedCMS server starting...
   Environment: dev
   Hot-reload: enabled
   Binding: 127.0.0.1:8333
✓ Server ready
```

### Edit Templates

**1. Open template:**
```bash
vim templates/layouts/knowledge/knowledge.mouse.jinja
```

**2. Make changes:**
```jinja
<h1>{{ "page.title" | text(lang) }}</h1>

{# Add subtitle #}
<p class="subtitle">{{ "page.subtitle" | text(lang) }}</p>
```

**3. Save file**

**4. Refresh browser** - changes visible immediately

**No server restart needed!**

---

## What Gets Reloaded?

### Automatically Reloaded

✅ **Template files (`.jinja`):**
- Layouts
- Components (atoms, molecules, organisms)
- Base templates
- Included templates

✅ **Template inheritance:**
- Changes to parent templates
- Changes to child templates

✅ **Includes:**
- Changes to included components

### NOT Automatically Reloaded

❌ **CSS files:**
- Requires asset rebuild: `reed build:assets`

❌ **Text content (`.reed/text.csv`):**
- Cache not invalidated automatically
- Restart server to reload

❌ **Server configuration:**
- Restart server: `reed server:restart`

❌ **Rust code:**
- Recompile: `cargo build`
- Restart server

---

## Performance Impact

### Development Mode

| Operation | Time | Overhead |
|-----------|------|----------|
| File stat check | ~1ms | +1ms |
| Template recompile (changed) | ~10ms | +10ms |
| Template render (cached) | < 1ms | 0ms |
| **Total (changed)** | ~11ms | +11ms |
| **Total (unchanged)** | ~2ms | +1ms |

**Acceptable for development** - instant visual feedback worth the overhead.

### Production Mode

| Operation | Time |
|-----------|------|
| Template render | < 1ms |
| **Total** | < 1ms |

**No overhead** - templates compiled once at startup.

---

## Best Practices

### Development

**Use hot-reload for fast iteration:**
```bash
# Terminal 1: Keep server running
reed server:io

# Terminal 2: Edit templates
vim templates/layouts/knowledge/knowledge.mouse.jinja

# Browser: Refresh to see changes (no restart!)
```

**Watch logs for errors:**
```bash
# Server shows template errors immediately
[ERROR] Template render failed: knowledge.mouse.jinja
        Syntax error at line 42: Unexpected '}'
```

**Separate CSS workflow:**
```bash
# CSS changes require rebuild
vim templates/layouts/knowledge/knowledge.mouse.css
reed build:assets

# Refresh browser
```

### Production

**Disable hot-reload:**
```env
ENVIRONMENT=prod
```

**Pre-compile templates:**
```bash
# Build production binary
cargo build --release

# Templates compiled and cached at startup
./target/release/reedcms server:start
```

**Monitor performance:**
```bash
# Check template render times
reed server:status
# Average render: 0.8ms
```

---

## Troubleshooting

### Changes Not Visible

**Check environment:**
```bash
echo $ENVIRONMENT
# Should be: dev
```

**Check server logs:**
```bash
reed server:logs --follow
# Look for: "Hot-reload: enabled"
```

**Hard refresh browser:**
```
Ctrl+Shift+R (Windows/Linux)
Cmd+Shift+R (macOS)
```

### Syntax Errors

**Server logs show errors:**
```
[ERROR] Template render failed
Template: layouts/knowledge/knowledge.mouse.jinja
Line: 42
Error: Unexpected token '}'
```

**Fix template and save:**
- No restart needed
- Next request will retry compilation

### Performance Issues in Dev

**Too slow?**
```bash
# Check if many templates being recompiled
reed server:logs | grep "Template recompile"

# Consider disabling for specific templates
# Or switch to production mode for testing
ENVIRONMENT=prod reed server:io
```

---

## Alternative: File Watcher (Future)

### Planned Enhancement

**Watch filesystem for changes:**
```rust
use notify::Watcher;

let (tx, rx) = channel();
let mut watcher = watcher(tx, Duration::from_secs(1))?;

watcher.watch("templates/", RecursiveMode::Recursive)?;

// Invalidate cache on file change
for event in rx {
    if let Event::Write(path) = event {
        template_cache.invalidate(&path);
    }
}
```

**Benefits:**
- Faster than file stat checks
- Can trigger asset rebuilds automatically
- Can reload text content automatically

---

## Comparison with Other Systems

| Feature | ReedCMS | Next.js | Laravel |
|---------|---------|---------|---------|
| Template hot-reload | ✅ Yes (MiniJinja) | ✅ Yes (Fast Refresh) | ✅ Yes (Blade) |
| Asset hot-reload | ❌ Manual rebuild | ✅ Yes (HMR) | ✅ Yes (Vite) |
| Overhead (dev) | ~1ms | ~50ms | ~10ms |
| Configuration | ENVIRONMENT=dev | Automatic | Automatic |

---

**See also:**
- [MiniJinja Integration](minijinja-integration.md) - Template engine
- [Server Commands](../04-cli-layer/server-commands.md) - Server control
- [Asset Layer](../08-asset-layer/) - CSS/JS bundling
